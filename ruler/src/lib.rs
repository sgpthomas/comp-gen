/*! `Ruler` is a framework for automatically inferring rewrite rules using equality saturation.
It uses equality saturation in two novel ways to scale the rule synthesis:
    1. to minimize the term space from which candidate rules are selected,
   and 2. to minimize the candidate rule space by removing redundant rules based on rules
   currently in the ruleset.
!*/
use egg::*;
use itertools::Itertools;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{
    borrow::{Borrow, Cow},
    collections::VecDeque,
};
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    time::Duration,
    time::Instant,
};
use std::{hash::BuildHasherDefault, sync::Arc};

mod convert_sexp;
mod derive;
mod equality;
mod util;

#[cfg(feature = "cli")]
pub mod cli;

/// Faster hashMap implementation used in rustc
pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

/// Faster hashSet implementation used in rustc
pub type HashSet<K> = rustc_hash::FxHashSet<K>;

/// IndexMap data implementation used in rustc
pub type IndexMap<K, V> =
    indexmap::IndexMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;

pub use equality::*;
pub use util::*;

/// Return the `i`th letter from the English alphabet.
pub fn letter(i: usize) -> &'static str {
    let alpha = "abcdefghijklmnopqrstuvwxyz";
    &alpha[i..i + 1]
}

/// Properties of cvecs in `Ruler`; currently onyl their length.
/// cvecs are stored as [eclass analysis data](https://docs.rs/egg/0.7.1/egg/trait.Analysis.html).
#[derive(Debug, Clone)]
pub struct SynthAnalysis {
    /// Length of cvec or characteristic vector.
    /// All cvecs have the same length.    
    pub cvec_len: usize,
}

impl Default for SynthAnalysis {
    fn default() -> Self {
        Self { cvec_len: 10 }
    }
}

/// Trait for defining a language for which `Ruler` will synthesize rewrites.
///
/// Every domain defines it own `Constant` type.
/// `eval` implements an interpreter for the domain. It returns a `Cvec` of length `cvec_len`
/// where each cvec element is computed using `eval`.
pub trait SynthLanguage:
    egg::Language + Send + Sync + Display + FromOp + 'static
{
    type Constant: Clone + Hash + Eq + Debug + Display;
    type Config: Clone;

    fn eval<'a, F>(&'a self, cvec_len: usize, f: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self>;

    fn to_var(&self) -> Option<Symbol>;
    fn mk_var(sym: egg::Symbol) -> Self;
    fn is_var(&self) -> bool {
        self.to_var().is_some()
    }

    fn to_constant(&self) -> Option<&Self::Constant>;
    fn mk_constant(c: Self::Constant) -> Option<Self>;
    fn is_constant(&self) -> bool {
        self.to_constant().is_some()
    }

    fn post_process(
        _synth: &SynthParams,
        report: Report<Self>,
    ) -> Report<Self> {
        report
    }

    /// Generalize a pattern
    fn generalize(
        expr: &RecExpr<Self>,
        map: &mut HashMap<Symbol, Var>,
    ) -> Pattern<Self> {
        let nodes: Vec<_> = expr
            .as_ref()
            .iter()
            .map(|n| match n.to_var() {
                Some(sym) => {
                    let var = if let Some(var) = map.get(&sym) {
                        *var
                    } else {
                        let var =
                            format!("?{}", letter(map.len())).parse().unwrap();
                        map.insert(sym, var);
                        var
                    };
                    ENodeOrVar::Var(var)
                }
                None => ENodeOrVar::ENode(n.clone()),
            })
            .collect();

        Pattern::from(PatternAst::from(nodes))
    }

    /// Instantiate a pattern
    fn instantiate(pattern: &Pattern<Self>) -> RecExpr<Self> {
        let nodes: Vec<_> = pattern
            .ast
            .as_ref()
            .iter()
            .map(|n| match n {
                ENodeOrVar::ENode(n) => n.clone(),
                ENodeOrVar::Var(v) => {
                    let s = v.to_string();
                    assert!(s.starts_with('?'));
                    Self::mk_var(s[1..].into())
                }
            })
            .collect();

        RecExpr::from(nodes)
    }

    /// Heuristics for ranking rewrites based on number of variables,
    /// constants, size of the `lhs` and `rhs`, total size of `lhs` and `rhs`,
    /// and number of ops.
    fn score(lhs: &Pattern<Self>, rhs: &Pattern<Self>) -> [i32; 5] {
        let sz_lhs = AstSize.cost_rec(&lhs.ast) as i32;
        let sz_rhs = AstSize.cost_rec(&rhs.ast) as i32;
        // let sz_max_pattern = sz_lhs.max(sz_rhs);

        // lhs.vars() and rhs.vars() is deduping
        // examples
        //   (- x x) => 0 --- 1 b/c x only var
        //   (- x 0) => x --- 1 b/c x only var
        //   (+ x y) => (+ y x) --- 2 b/c x, y only vars
        let mut var_set: HashSet<Var> = Default::default();
        var_set.extend(lhs.vars());
        var_set.extend(rhs.vars());
        let n_vars_rule = var_set.len() as i32;

        let mut op_set: HashSet<String> = Default::default();
        for node in lhs.ast.as_ref().iter().chain(rhs.ast.as_ref()) {
            if !node.is_leaf() {
                // op_set.insert(node.display_op().to_string());
                op_set.insert(format!("{}", node));
            }
        }
        let n_ops = op_set.len() as i32;

        let n_consts = lhs
            .ast
            .as_ref()
            .iter()
            .chain(rhs.ast.as_ref())
            .filter(|n| match n {
                ENodeOrVar::ENode(n) => n.is_constant(),
                ENodeOrVar::Var(_) => false,
            })
            .count() as i32;

        // (-sz_max_pattern, n_vars_rule)
        [
            n_vars_rule,
            -n_consts,
            -i32::max(sz_lhs, sz_rhs),
            // -i32::min(sz_lhs, sz_rhs),
            -(sz_lhs + sz_rhs),
            -n_ops,
            // 0
        ]
    }

    /// Initialize an egraph with variables and interesting constants from the domain.
    fn init_synth(synth: &mut Synthesizer<Self, Uninit>);

    /// Layer wise term enumeration in the egraph.
    fn make_layer<'a>(
        ids: Vec<Id>,
        egraph: &'a Synthesizer<Self, Init>,
        iter: usize,
    ) -> Box<dyn Iterator<Item = Self> + 'a>;
    // fn make_layer(synth: &Synthesizer<Self>, iter: usize) -> Box<dyn Iterator<Item = Self> + '_>;
    // fn make_layer(synth: &Synthesizer<Self>, iter: usize) -> Vec<Self>;

    /// Given a , `ctx`, i.e., mapping from variables to cvecs, evaluate a pattern, `pat`,
    /// on each element of the cvec.
    fn eval_pattern(
        pat: &Pattern<Self>,
        ctx: &HashMap<Var, CVec<Self>>,
        cvec_len: usize,
    ) -> CVec<Self> {
        let mut buf: Vec<Cow<CVec<Self>>> = vec![];
        for enode in pat.ast.as_ref().iter() {
            match enode {
                ENodeOrVar::ENode(enode) => {
                    let cvec = enode
                        .eval(cvec_len, |id| buf[usize::from(*id)].borrow());
                    buf.push(Cow::Owned(cvec));
                }
                ENodeOrVar::Var(var) => {
                    buf.push(Cow::Borrowed(&ctx[var]));
                }
            }
        }
        buf.pop().unwrap().into_owned()
    }

    /// Domain specific rule validation.
    fn is_valid(
        synth: &mut Synthesizer<Self, Init>,
        lhs: &Pattern<Self>,
        rhs: &Pattern<Self>,
    ) -> bool;

    /// helper functions to convert CVC4 rewrites to Ruler's rule syntax.
    fn convert_parse(s: &str) -> RecExpr<Self> {
        s.parse().unwrap()
    }

    /// convert CVC4 rewrites to Ruler's rule syntax.
    fn convert_eq(s: &str) -> Option<Equality<Self>> {
        use symbolic_expressions::{parser::parse_str, Sexp};
        let sexp = parse_str(s).unwrap();
        let list = sexp.list().unwrap();
        assert!(
            list[0] == Sexp::String("candidate-rewrite".into())
                || list[0] == Sexp::String("rewrite".into())
        );
        let l = Self::convert_parse(&list[1].to_string());
        let r = Self::convert_parse(&list[2].to_string());
        Equality::new(&l, &r)
    }
}

#[cfg(feature = "cli")]
pub trait Main {
    fn main();
}

#[cfg(feature = "cli")]
impl<T: SynthLanguage> Main for T
where
    T::Config: Default,
{
    /// Entry point. Use the `synth` argument from the command line
    /// for rule synthesis.
    fn main() {
        let _ = env_logger::builder().try_init();
        match cli::Command::cli() {
            cli::Command::Synth(params) => {
                let params: SynthParams = params.into();
                let outfile = params.outfile.clone();
                let syn = Synthesizer::<Self, _>::new(params.clone()).init();
                let report: Report<Self> =
                    Self::post_process(&params, syn.run());
                let file = std::fs::File::create(&outfile)
                    .unwrap_or_else(|_| panic!("Failed to open '{}'", outfile));
                serde_json::to_writer_pretty(file, &report)
                    .expect("failed to write json");
            }
            cli::Command::Derive(params) => {
                derive::derive::<Self>(params.into())
            }
            cli::Command::ConvertSexp(params) => {
                convert_sexp::convert::<Self>(params.into())
            }
        }
    }
}

#[derive(Clone)]
pub struct Uninit;
#[derive(Clone)]
pub struct Init;

/// A synthesizer for a given [SynthLanguage].
#[derive(Clone)]
pub struct Synthesizer<L: SynthLanguage, State> {
    pub params: SynthParams,
    pub lang_config: L::Config,
    pub rng: Pcg64,
    pub egraph: EGraph<L, SynthAnalysis>,
    initial_egraph: EGraph<L, SynthAnalysis>,
    pub equalities: EqualityMap<L>,
    pub smt_unknown: usize,
    start_time: Instant,
    phantom_state: PhantomData<State>,
}

impl<L: SynthLanguage> Synthesizer<L, Uninit> {
    pub fn new_with_data(params: SynthParams, data: L::Config) -> Self {
        Synthesizer {
            rng: Pcg64::seed_from_u64(params.seed),
            egraph: Default::default(),
            initial_egraph: Default::default(),
            equalities: Default::default(),
            smt_unknown: 0,
            params,
            start_time: Instant::now(),
            lang_config: data,
            phantom_state: PhantomData,
        }
    }

    pub fn init(mut self) -> Synthesizer<L, Init> {
        L::init_synth(&mut self);
        self.initial_egraph = self.egraph.clone();
        Synthesizer {
            phantom_state: PhantomData,
            // copy the rest of the fields
            params: self.params,
            lang_config: self.lang_config,
            rng: self.rng,
            egraph: self.egraph,
            initial_egraph: self.initial_egraph,
            equalities: self.equalities,
            smt_unknown: self.smt_unknown,
            start_time: self.start_time,
        }
    }
}

impl<L: SynthLanguage> Synthesizer<L, Uninit>
where
    L::Config: Default,
{
    /// Initialize all the arguments of the [Synthesizer].
    pub fn new(params: SynthParams) -> Self {
        Self::new_with_data(params, L::Config::default())
    }
}

impl<L: SynthLanguage> Synthesizer<L, Init> {
    fn check_time(&self) -> bool {
        let t = self.start_time;
        self.params.abs_timeout > 0
            && t.elapsed() > Duration::from_secs(self.params.abs_timeout as u64)
    }

    fn time_left(&self) -> Duration {
        let limit = Duration::from_secs(self.params.abs_timeout as u64);
        if self.start_time.elapsed() > limit {
            Duration::ZERO
        } else {
            Duration::from_secs(self.params.abs_timeout as u64)
                - self.start_time.elapsed()
        }
    }

    /// Get the eclass ids for all eclasses in the egraph.
    pub fn ids(&self) -> impl '_ + Iterator<Item = Id> {
        self.egraph.classes().map(|c| c.id)
    }

    /// Create a [runner](https://docs.rs/egg/0.6.0/egg/struct.Runner.html).
    fn mk_runner(
        &self,
        mut egraph: EGraph<L, SynthAnalysis>,
    ) -> Runner<L, SynthAnalysis, ()> {
        let node_limit = self.params.eqsat_node_limit;

        let mut runner = Runner::default()
            .with_node_limit(usize::MAX)
            .with_hook(move |r| {
                let size = r.egraph.total_number_of_nodes();
                if size > node_limit {
                    Err(format!("Node limit: {}", size))
                } else {
                    Ok(())
                }
            })
            .with_node_limit(self.params.eqsat_node_limit * 2)
            .with_iter_limit(self.params.eqsat_iter_limit)
            // .with_time_limit(Duration::from_secs(self.params.eqsat_time_limit))
            .with_time_limit(self.time_left())
            .with_scheduler(SimpleScheduler);
        runner = if self.params.enable_explanations {
            runner.with_explanations_enabled()
        } else {
            runner
        };
        runner = if self.params.no_conditionals {
            egraph.analysis.cvec_len = 0;
            for c in egraph.classes_mut() {
                c.data.cvec.truncate(0);
            }
            runner.with_egraph(egraph).with_hook(|r| {
                for c in r.egraph.classes_mut() {
                    if c.nodes.iter().any(|n| n.is_constant()) {
                        c.nodes.retain(|n| n.is_constant());
                    }
                }
                Ok(())
            })
        } else {
            runner.with_egraph(egraph)
        };
        runner
    }

    /// Apply current ruleset to the term egraph to minimize the term space.
    #[inline(never)]
    fn run_rewrites(&mut self) -> EGraph<L, SynthAnalysis> {
        // run the rewrites
        log::info!("running eqsat with {} rules", self.equalities.len());
        let start = Instant::now();
        let rewrites: Vec<&Rewrite<L, SynthAnalysis>> = self
            .equalities
            .values()
            .flat_map(|eq| &eq.rewrites)
            .collect();

        let mut runner = self.mk_runner(self.egraph.clone());
        runner = runner.run(rewrites);

        log::info!("{:?} collecting unions...", runner.stop_reason.unwrap());
        // update the clean egraph based on any unions that happened
        let mut found_unions = HashMap::default();
        for id in self.ids() {
            let id2 = runner.egraph.find(id);
            found_unions.entry(id2).or_insert(vec![]).push(id);
        }
        for ids in found_unions.values() {
            for win in ids.windows(2) {
                self.egraph.union(win[0], win[1]);
            }
        }
        runner.egraph.rebuild();

        // for eclass in runner.egraph.classes() {
        //     if eclass.data.cvec.iter().any(|x| x.is_some()) {
        //         eprintln!("eclass: {} {:?}", eclass.id, eclass.data.cvec);
        //     }
        // }

        log::info!(
            "Ran {} rules in {:?}",
            self.equalities.len(),
            start.elapsed()
        );
        runner.egraph
    }

    /// Generate potential rewrite rule candidates by cvec_matching.
    ///
    /// Note that this is a pair-wise matcher which is invoked when conditional
    /// rewrite rule inference is enabled.
    #[inline(never)]
    fn cvec_match_pair_wise(&self) -> EqualityMap<L> {
        let mut by_cvec: IndexMap<CVec<L>, Vec<Id>> = IndexMap::default();

        let not_all_nones = self.ids().filter(|id| {
            !&self.egraph[*id].data.cvec.iter().all(|v| v == &None)
        });
        for id in not_all_nones {
            let class = &self.egraph[id];
            let cvec = vec![class.data.cvec[0].clone()];
            by_cvec.entry(cvec).or_default().push(class.id);
        }

        log::info!("(cvec_match_pair_wise) # unique cvecs: {}", by_cvec.len());

        let mut new_eqs = EqualityMap::default();
        let extract = Extractor::new(&self.egraph, AstSize);

        let compare = |cvec1: &CVec<L>, cvec2: &CVec<L>| -> bool {
            // keep track of number of pairs that match
            // we need at least 1 pair to say that the cvecs
            // are equal
            let mut count = 0;
            for tup in cvec1.iter().zip(cvec2) {
                count += match tup {
                    (Some(a), Some(b)) if a != b => return false,
                    (Some(a), Some(b)) if a == b => 1,
                    _ => 0,
                };
            }
            count > 0
        };

        log::info!("CVec Loop");
        for ids in by_cvec.values() {
            let mut ids = ids.iter().copied();
            while let Some(id1) = ids.next() {
                for id2 in ids.clone() {
                    if compare(
                        &self.egraph[id1].data.cvec,
                        &self.egraph[id2].data.cvec,
                    ) {
                        let (_, e1) = extract.find_best(id1);
                        let (_, e2) = extract.find_best(id2);
                        if let Some(mut eq) = Equality::new(&e1, &e2) {
                            log::debug!("  Candidate {}", eq);
                            eq.ids = Some((id1, id2));
                            new_eqs.insert(eq.name.clone(), eq);
                        }
                    }
                }
            }
        }
        log::info!("done");

        new_eqs.retain(|k, _v| !self.equalities.contains_key(k));
        new_eqs
    }

    /// Generate potential rewrite rule candidates by cvec_matching.
    /// This is a more efficient implementation that hashes the eclasses by their cvecs.
    ///
    /// Note that this is only used when conditional rewrite rule inference is disabled.
    #[inline(never)]
    fn cvec_match(&self) -> (EqualityMap<L>, Vec<Vec<Id>>) {
        // build the cvec matching data structure
        let mut by_cvec: IndexMap<&CVec<L>, Vec<Id>> = IndexMap::default();

        for id in self.ids() {
            let class = &self.egraph[id];
            if class.data.is_defined() {
                by_cvec.entry(&class.data.cvec).or_default().push(class.id);
            }
        }

        log::info!("(cvec_match) # unique cvecs: {}", by_cvec.len());

        let mut new_eqs = EqualityMap::default();
        let extract = Extractor::new(&self.egraph, AstSize);
        for ids in by_cvec.values() {
            if self.params.linear_cvec_matching || ids.len() > 0 {
                let mut terms_ids: Vec<_> =
                    ids.iter().map(|&id| (extract.find_best(id), id)).collect();
                terms_ids.sort_by_key(|x| x.0 .0);
                let ((_c1, e1), id1) = terms_ids.remove(0);
                for ((_c2, e2), id2) in terms_ids {
                    if let Some(mut eq) = Equality::new(&e1, &e2) {
                        log::debug!("  Candidate {}", eq);
                        eq.ids = Some((id1, id2));
                        new_eqs.insert(eq.name.clone(), eq);
                    }
                }
            } else {
                let mut id_iter = ids.iter();
                while let Some(&id1) = id_iter.next() {
                    let (_, e1) = extract.find_best(id1);
                    for &id2 in id_iter.clone() {
                        let (_, e2) = extract.find_best(id2);
                        if let Some(mut eq) = Equality::new(&e1, &e2) {
                            log::debug!("  Candidate {}", eq);
                            eq.ids = Some((id1, id2));
                            new_eqs.insert(eq.name.clone(), eq);
                        }
                    }
                }
            }
        }

        new_eqs.retain(|k, _v| !self.equalities.contains_key(k));
        (new_eqs, by_cvec.into_iter().map(|pair| pair.1).collect())
    }

    /// Top level function for rule synthesis.
    /// This corresponds to `Figure 4` in the Ruler paper, where
    /// all the key components of `Ruler`
    /// (e.g., `make_layer`, `run_rewrites`, `cvec_match`, `choose_eqs`) are invoked.
    pub fn run(mut self) -> Report<L> {
        // normalize some params
        if self.params.rules_to_take == 0 {
            self.params.rules_to_take = usize::MAX;
        }
        if self.params.chunk_size == 0 {
            self.params.chunk_size = usize::MAX;
        }

        let mut poison_rules: HashSet<Equality<L>> = HashSet::default();
        let t = Instant::now();
        assert!(self.params.iters > 0);
        'outer: for iter in 1..=self.params.iters {
            log::info!("[[[ Iteration {} ]]]", iter);

            let synth_copy = self.clone();
            let layer =
                L::make_layer(self.ids().collect_vec(), &synth_copy, iter)
                    .filter(|n| !n.all(|id| synth_copy.egraph[id].data.exact));

            // using a filter instead of retain, because make_layer returns an iter
            // and not a vector
            // layer.retain(|n| !n.all(|id| self.egraph[id].data.exact));

            // log::info!("{}", iter > self.params.no_constants_above_iter);
            // if iter > self.params.no_constants_above_iter {
            //     let mut extract = Extractor::new(&self.egraph, NumberOfOps);

            //     // maps ids to n_ops
            //     let has_constants: HashSet<Id> = self
            //         .ids()
            //         .filter_map(|id| {
            //             let (_cost, best) = extract.find_best(id);
            //             if best.as_ref().iter().any(|n| n.is_constant())
            //                 Some(id)
            //             } else {
            //                 None
            //             }
            //         })
            //         .collect();

            //     layer.retain(|n| n.all(|id| !has_constants.contains(&id)));
            // }

            // log::info!("Made layer of {} nodes", layer.len());

            log::info!(
                "Made layer! Using chunk size: {}",
                self.params.chunk_size
            );
            for chunk in &layer.chunks(self.params.chunk_size) {
                log::info!(
                    "egraph n={}, e={}",
                    self.egraph.total_size(),
                    self.egraph.number_of_classes(),
                );
                for node in chunk {
                    if self.check_time() {
                        break 'outer;
                    }
                    self.egraph.add(node);
                }
                'inner: loop {
                    // abort if it's been longer than abs_timeout
                    if self.check_time() {
                        break 'outer;
                    }

                    let run_rewrites_before = Instant::now();
                    if !self.params.no_run_rewrites {
                        self.run_rewrites();
                    }
                    let run_rewrites =
                        run_rewrites_before.elapsed().as_secs_f64();

                    let rule_discovery_before = Instant::now();
                    log::info!(
                        "cvec matching... (no_conditionals: {})",
                        self.params.no_conditionals
                    );
                    let (new_eqs, _) = if self.params.no_conditionals {
                        self.cvec_match()
                    } else {
                        (self.cvec_match_pair_wise(), vec![])
                    };
                    let rule_discovery =
                        rule_discovery_before.elapsed().as_secs_f64();
                    log::info!("{} candidate eqs", new_eqs.len());

                    let new_eqs: EqualityMap<L> = new_eqs
                        .into_iter()
                        .filter(|eq| !poison_rules.contains(&eq.1))
                        .collect();

                    log::info!(
                        "egraph n={}, e={}",
                        self.egraph.total_size(),
                        self.egraph.number_of_classes(),
                    );

                    let rule_minimize_before = Instant::now();
                    let (eqs, bads) = if self.params.minimize {
                        self.minimize(new_eqs)
                    } else {
                        log::info!("choose_eqs: {}", new_eqs.len());
                        self.choose_eqs(new_eqs)
                    };

                    let rule_minimize =
                        rule_minimize_before.elapsed().as_secs_f64();

                    for bad in bads {
                        let s = format!("{}", bad.0);
                        // make sure that none of the rules contain a quote
                        if s.contains("!!!") || s.contains("\"") {
                            log::info!("culprit: {}", bad.0);
                            panic!();
                        }
                        log::debug!("adding {} to poison set", bad.0);
                        poison_rules.insert(bad.1);
                    }
                    if eqs.is_empty() {
                        log::info!("Stopping early, no eqs");
                        break 'inner;
                    }

                    log::info!("Chose {} good rules", eqs.len());
                    for eq in eqs.values() {
                        log::debug!("  {}", eq);
                        if !self.params.no_run_rewrites {
                            assert!(!self.equalities.contains_key(&eq.name));
                            if let Some((i, j)) = eq.ids {
                                self.egraph.union(i, j);
                            }
                        }
                    }

                    self.equalities.extend(eqs);

                    // TODO check formatting for Learned...
                    log::info!("Time taken in... run_rewrites: {}, rule discovery: {}, rule minimization: {}",
                    run_rewrites,  rule_discovery, rule_minimize);
                    // For the no-conditional case which returns
                    // a non-empty list of ids that have the same cvec,
                    // won't this cause eclasses to merge even if the rule is actually not valid?
                    if self.params.minimize
                        || self.params.rules_to_take == usize::MAX
                    {
                        log::info!("Stopping early, took all eqs");
                        break 'inner;
                    }
                }
            }
        }

        let time = t.elapsed().as_secs_f64();
        let num_rules = self.equalities.len();
        let mut eqs: Vec<_> = self
            .equalities
            .clone()
            .into_iter()
            .map(|(_, eq)| eq)
            .collect();
        eqs.sort_by_key(|eq| eq.score());
        eqs.reverse();

        // final run_rewrites
        if self.params.do_final_run {
            let old =
                std::mem::replace(&mut self.params.no_conditionals, false);
            let rws = self.equalities.values().flat_map(|eq| &eq.rewrites);
            let final_runner = self.mk_runner(self.egraph.clone());
            final_runner.run(rws);
            self.params.no_conditionals = old;
        }

        println!("Learned {} rules in {:?}", num_rules, time);
        for eq in &eqs {
            // println!("  {:?}   {}", eq.score(), eq);
            println!("{}", eq);
        }
        println!("Learned {} rules in {:?}", num_rules, time);
        Report {
            params: self.params,
            // lang_config: self.lang_config,
            time,
            num_rules,
            eqs,
            smt_unknown: self.smt_unknown,
        }
    }
}

/// Reports for each run of Ruler.
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "L: SynthLanguage")]
pub struct Report<L: SynthLanguage> {
    pub params: SynthParams,
    // pub lang_config: L::Config,
    pub time: f64,
    pub num_rules: usize,
    pub smt_unknown: usize,
    pub eqs: Vec<Equality<L>>,
}

#[derive(Serialize, Deserialize)]
#[serde(bound = "L: SynthLanguage")]
struct SlimReport<L: SynthLanguage> {
    time: f64,
    eqs: Vec<Equality<L>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthParams {
    pub seed: u64,
    pub n_samples: usize,
    pub variables: usize,
    pub abs_timeout: usize,
    pub iters: usize,
    pub rules_to_take: usize,
    pub chunk_size: usize,
    pub minimize: bool,
    pub no_constants_above_iter: usize,
    pub no_conditionals: bool,
    pub no_run_rewrites: bool,
    pub linear_cvec_matching: bool,
    pub outfile: String,
    pub eqsat_node_limit: usize,
    pub eqsat_iter_limit: usize,
    pub eqsat_time_limit: u64,
    pub important_cvec_offsets: u32,
    pub str_int_variables: usize,
    pub complete_cvec: bool,
    pub no_xor: bool,
    pub no_shift: bool,
    pub num_fuzz: usize,
    pub use_smt: bool,
    pub do_final_run: bool,
    pub enable_explanations: bool,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            seed: 0,
            n_samples: 0,
            variables: 3,
            abs_timeout: 120,
            iters: 1,
            rules_to_take: 0,
            chunk_size: 100000,
            minimize: false,
            no_constants_above_iter: 999999,
            no_conditionals: false,
            no_run_rewrites: false,
            linear_cvec_matching: false,
            outfile: "out.json".to_string(),
            eqsat_node_limit: 300000,
            eqsat_iter_limit: 2,
            eqsat_time_limit: 60,
            important_cvec_offsets: 5,
            str_int_variables: 1,
            complete_cvec: false,
            no_xor: false,
            no_shift: false,
            num_fuzz: 0,
            use_smt: false,
            do_final_run: false,
            enable_explanations: false,
        }
    }
}

impl SynthParams {
    pub fn from_path(
        path: &std::path::PathBuf,
    ) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let config = serde_json::from_reader(file)?;
        Ok(config)
    }
}

/// A mapping from a name to an `Equality`.
pub type EqualityMap<L> = IndexMap<Arc<str>, Equality<L>>;

/// A CVec is a data structure that stores the result of evaluating a term on concrete inputs.
pub type CVec<L> = Vec<Option<<L as SynthLanguage>::Constant>>;

/// Simple macro for generating cvecs for unary, binary, and ternary ops.
#[macro_export]
macro_rules! map {
    ($get:ident, $a:ident => $body:expr) => {
        $get($a)
            .iter()
            .map(|a| match a {
                Some($a) => $body,
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    ($get:ident, $a:ident, $b:ident => $body:expr) => {
        $get($a)
            .iter()
            .zip($get($b).iter())
            .map(|tup| match tup {
                (Some($a), Some($b)) => $body,
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    ($get:ident, $a:ident, $b:ident, $c:ident => $body:expr) => {
        $get($a)
            .iter()
            .zip($get($b).iter())
            .zip($get($c).iter())
            .map(|tup| match tup {
                ((Some($a), Some($b)), Some($c)) => $body,
                _ => None,
            })
            .collect::<Vec<_>>()
    };
}

/// The Signature represents eclass analysis data.
#[derive(Debug, Clone)]
pub struct Signature<L: SynthLanguage> {
    pub cvec: CVec<L>,
    pub exact: bool,
    pub vars: Vec<egg::Symbol>,
}

impl<L: SynthLanguage> Signature<L> {
    /// A cvec is defined either it is empty or has at least one `Some` value.
    pub fn is_defined(&self) -> bool {
        self.cvec.is_empty() || self.cvec.iter().any(|v| v.is_some())
    }
}

// unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
//     let const_ptr = reference as *const T;
//     let mut_ptr = const_ptr as *mut T;
//     &mut *mut_ptr
// }

impl<L: SynthLanguage> egg::Analysis<L> for SynthAnalysis {
    type Data = Signature<L>;

    fn pre_union(
        egraph: &EGraph<L, Self>,
        id1: Id,
        id2: Id,
        justification: &Option<egg::Justification>,
    ) {
        for (val1, val2) in egraph[id1]
            .data
            .cvec
            .iter()
            .zip(egraph[id2].data.cvec.iter())
        {
            match (val1, val2) {
                (Some(x), Some(y)) if x != y => {
                    let extractor = egg::Extractor::new(egraph, egg::AstSize);
                    let (_, prog1) = extractor.find_best(id1);
                    let (_, prog2) = extractor.find_best(id2);

                    // let mut_egraph = unsafe { very_bad_function(&egraph) };
                    // mut_egraph.explain_equivalence(&prog1, &prog2);

                    log::info!("{} <=> {}", prog1.pretty(80), prog2.pretty(80));
                    log::info!("cvec1: {:?}", egraph[id1].data.cvec);
                    log::info!("cvec2: {:?}", egraph[id2].data.cvec);
                    log::info!("just: {justification:?}");
                }
                _ => (),
            }
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        let mut changed_a = false;
        let mut changed_b = false;

        if !to.cvec.is_empty() && !from.cvec.is_empty() {
            for i in 0..to.cvec.len() {
                match (to.cvec[i].clone(), from.cvec[i].clone()) {
                    (None, Some(_)) => {
                        changed_a = true;
                        to.cvec[i] = from.cvec[i].clone();
                    }
                    (Some(x), Some(y)) => {
                        assert_eq!(x, y, "cvecs do not match at index {}", i)
                    }
                    (Some(_), None) => {
                        changed_b = true;
                    }
                    _ => (),
                }
            }

            to.exact |= from.exact;
        }

        // merge vars by appending arr
        match (to.vars.is_empty(), from.vars.is_empty()) {
            // both are empty and so don't make any changes
            (true, true) => (),
            // a is empty, b isn't. set a = b. change a (not b because b is not equal to a)
            (true, false) => {
                to.vars = from.vars;
                changed_a = true;
            }
            // a isn't empty, b is empty. a is already a, so don't do anything
            (false, true) => (),
            // neither are empty. extend to.vars with from.vars. both change
            (false, false) => {
                to.vars.extend(from.vars.into_iter());
                changed_a = true;
                changed_b = true;
            }
        }

        DidMerge(changed_a, changed_b)
    }

    fn make(egraph: &EGraph<L, Self>, enode: &L) -> Self::Data {
        let get_cvec = |i: &Id| &egraph[*i].data.cvec;
        Signature {
            cvec: enode.eval(egraph.analysis.cvec_len, get_cvec),
            exact: !enode.is_var() && enode.all(|i| egraph[i].data.exact),
            vars: enode.to_var().into_iter().collect_vec(),
        }
    }

    fn modify(egraph: &mut EGraph<L, Self>, id: Id) {
        let sig = &egraph[id].data;
        if sig.exact {
            let first = sig.cvec.iter().find_map(|x| x.as_ref());
            if let Some(first) = first {
                if let Some(enode) = L::mk_constant(first.clone()) {
                    let added = egraph.add(enode);
                    egraph.union(id, added);
                }
            }
        }
    }
}

impl<L: SynthLanguage> Synthesizer<L, Init> {
    /// Shrink the candidate space.
    #[inline(never)]
    fn shrink(
        &mut self,
        mut new_eqs: EqualityMap<L>,
        step: usize,
        should_validate: bool,
    ) -> (EqualityMap<L>, EqualityMap<L>) {
        let mut keepers = EqualityMap::default();
        let mut bads = EqualityMap::default();
        let initial_len = new_eqs.len();
        while !new_eqs.is_empty() {
            if self.check_time() {
                break;
            }

            // best are last
            new_eqs.sort_by(|_, eq1, _, eq2| eq1.score().cmp(&eq2.score()));

            // take step valid rules from the end of new_eqs
            let mut took = 0;
            while let Some((name, eq)) = new_eqs.pop() {
                if self.check_time() {
                    break;
                }

                if should_validate {
                    let rule_validation = Instant::now();
                    let valid = L::is_valid(self, &eq.lhs, &eq.rhs);
                    log::debug!(
                        "Time taken in validation: {}",
                        rule_validation.elapsed().as_secs_f64()
                    );

                    if valid {
                        let old = keepers.insert(name, eq);
                        took += old.is_none() as usize;
                    } else {
                        bads.insert(name, eq);
                    }
                } else {
                    let old = keepers.insert(name, eq);
                    took += old.is_none() as usize;
                }

                if took >= step {
                    break;
                }
            }

            if new_eqs.is_empty() {
                break;
            }

            if keepers.len() >= self.params.rules_to_take {
                keepers.truncate(self.params.rules_to_take);
                break;
            }

            log::info!("Shrinking with {} keepers", keepers.len());

            let rewrites = self
                .equalities
                .values()
                .flat_map(|eq| &eq.rewrites)
                .chain(keepers.values().flat_map(|eq| &eq.rewrites));

            log::info!("# rewrites: {}", rewrites.clone().count());
            // for eq in rewrites.clone() {
            //     log::info!("rewrites: {}", eq.name);
            // }

            let mut runner = self.mk_runner(self.initial_egraph.clone());
            for candidate_eq in new_eqs.values() {
                runner = runner.with_expr(&L::instantiate(&candidate_eq.lhs));
                runner = runner.with_expr(&L::instantiate(&candidate_eq.rhs));
            }

            runner = runner.run(rewrites);
            log::info!(
                "Stopping {:?}, {:?}",
                runner.stop_reason.clone().unwrap(),
                runner.iterations.len()
            );

            let old_len = new_eqs.len();
            let extract = Extractor::new(&runner.egraph, AstSize);
            new_eqs.clear();
            for ids in runner.roots.chunks(2) {
                if runner.egraph.find(ids[0]) != runner.egraph.find(ids[1]) {
                    let left = extract.find_best(ids[0]).1;
                    let right = extract.find_best(ids[1]).1;
                    if let Some(eq) = Equality::new(&left, &right) {
                        if !self.equalities.contains_key(&eq.name) {
                            new_eqs.insert(eq.name.clone(), eq);
                        }
                    }
                }
            }

            log::info!(
                "Minimizing... threw away {} rules, {} / {} remain",
                old_len - new_eqs.len(),
                new_eqs.len(),
                initial_len,
            );
        }
        if self.params.rules_to_take == usize::MAX && !new_eqs.is_empty() {
            // assert!(new_eqs.is_empty());
            // eprintln!("{:?}", new_eqs);
            eprintln!("New eqs not empty");
        }
        (keepers, bads)
    }

    /// Apply rewrites rules as they are being inferred, to minimize the candidate space.
    #[inline(never)]
    fn choose_eqs(
        &mut self,
        mut new_eqs: EqualityMap<L>,
    ) -> (EqualityMap<L>, EqualityMap<L>) {
        let mut bads = EqualityMap::default();
        let mut should_validate = true;
        for step in vec![100, 10, 1] {
            if self.check_time() {
                break;
            }

            if self.params.rules_to_take < step {
                continue;
            }
            let n_rules = usize::min(self.params.rules_to_take, new_eqs.len());
            if step < 10 && n_rules > 200 {
                break;
            }
            let (n, b) = self.shrink(new_eqs, step, should_validate);
            new_eqs = n;
            bads.extend(b);

            // if we taking all the rules, we don't need to validate anymore
            if self.params.rules_to_take == usize::MAX {
                should_validate = false;
            }
        }
        (new_eqs, bads)
    }

    /// Alternate minimization algorithm which is currently not used.
    #[inline(never)]
    fn minimize(
        &mut self,
        new_eqs: EqualityMap<L>,
    ) -> (EqualityMap<L>, EqualityMap<L>) {
        assert!(self.params.minimize);

        let t = Instant::now();

        let rule_validation = Instant::now();
        let (new_eqs, bads): (EqualityMap<L>, EqualityMap<L>) = new_eqs
            .into_iter()
            .partition(|(_name, eq)| L::is_valid(self, &eq.lhs, &eq.rhs));

        log::debug!(
            "Time taken in validation: {}",
            rule_validation.elapsed().as_secs_f64()
        );

        let n_new_eqs = new_eqs.len();
        log::info!("Minimizing {} rules...", n_new_eqs);
        let mut flat: VecDeque<Equality<L>> =
            new_eqs.into_iter().map(|(_, eq)| eq).collect();
        let mut test = vec![];

        for mut n_chunks in (2..).map(|i| 1 << i) {
            if n_chunks > flat.len() {
                if n_chunks >= flat.len() * 2 {
                    break;
                }
                n_chunks = flat.len();
            }

            log::info!("n chunks {}", n_chunks);

            let mut chunk_sizes = vec![flat.len() / n_chunks; n_chunks];
            for i in 0..(flat.len() % n_chunks) {
                chunk_sizes[i] += 1;
            }
            assert_eq!(flat.len(), chunk_sizes.iter().sum::<usize>());

            for size in chunk_sizes {
                let n_new_eqs_this_loop = flat.len();
                test.clear();
                for _ in 0..size {
                    test.push(flat.pop_front().unwrap());
                }
                assert_eq!(test.len(), size);

                log::info!("chunk size {}, flat {}", size, flat.len());

                let mut rewrites: Vec<_> = self
                    .equalities
                    .values()
                    .flat_map(|eq| &eq.rewrites)
                    .chain(flat.iter().flat_map(|eq| &eq.rewrites))
                    .collect();

                rewrites.sort_by_key(|rw| rw.name);
                rewrites.dedup_by_key(|rw| rw.name);

                let mut runner = self.mk_runner(self.initial_egraph.clone());

                for candidate_eq in &test {
                    runner =
                        runner.with_expr(&L::instantiate(&candidate_eq.lhs));
                    runner =
                        runner.with_expr(&L::instantiate(&candidate_eq.rhs));
                }

                runner = runner.run(rewrites);

                let extract = Extractor::new(&runner.egraph, AstSize);
                flat.extend(runner.roots.chunks(2).zip(&test).filter_map(
                    |(ids, eq)| {
                        if runner.egraph.find(ids[0])
                            == runner.egraph.find(ids[1])
                        {
                            None
                        } else {
                            let lhs = extract.find_best(ids[0]).1;
                            let rhs = extract.find_best(ids[1]).1;
                            // TODO get ids so they can be unioned by `run`
                            if let Some(mut eq2) = Equality::new(&lhs, &rhs) {
                                eq2.ids = eq.ids;
                                Some(eq2)
                            } else {
                                None
                            }
                        }
                    },
                ));

                log::info!(
                    "Minimizing... threw away {} rules, {} / {} remain",
                    n_new_eqs_this_loop - flat.len(),
                    flat.len(),
                    n_new_eqs
                );
            }
        }

        log::info!(
            "Minimized {}->{} rules in {:?}",
            n_new_eqs,
            flat.len(),
            t.elapsed()
        );

        (
            flat.into_iter().map(|eq| (eq.name.clone(), eq)).collect(),
            bads,
        )
    }
}

/// An alternate cost function that computes the number of operators in an term.
pub struct NumberOfOps;
impl<L: Language> egg::CostFunction<L> for NumberOfOps {
    type Cost = usize;
    fn cost<C>(&mut self, enode: &L, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        if enode.is_leaf() {
            0
        } else {
            enode.fold(1, |sum, id| sum + costs(id))
        }
    }
}

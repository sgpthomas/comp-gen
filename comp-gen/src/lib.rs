pub mod config;

use egg::rewrite as rw;
use log::{debug, info};
pub use ruler;
use std::io::Write;
use std::marker::PhantomData;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub trait Interpreter {
    type Env: Default;
    type Res;

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res;
    fn eval(&self) -> Self::Res {
        self.eval_with_env(&mut Self::Env::default())
    }
}

pub trait ToRecExpr<T> {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<T>) -> egg::Id;
}

pub trait FromPattern: Sized {
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self>;
}

pub trait CostMetrics<L, N>: egg::CostFunction<L>
where
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
{
    fn cost_differential(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
    fn cost_average(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
    fn all(&mut self, r: &egg::Rewrite<L, N>) -> CostMetric<L, N, Self>
    where
        Self: Sized,
    {
        CostMetric {
            cd: self.cost_differential(r),
            ca: self.cost_average(r),
            phantom: PhantomData,
        }
    }
}

pub struct CostMetric<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    pub cd: C::Cost,
    pub ca: C::Cost,
    phantom: PhantomData<N>,
}

/// Structure that defines a rule phase.
struct Phase<
    L: egg::Language + egg::FromOp + Send + Sync + FromPattern + 'static,
    N: egg::Analysis<L> + Default + Clone,
    // C: egg::CostFunction<L> + Clone,
> {
    name: String,
    // filter: Box<dyn Fn(CostMetric<L, N, C>) -> bool>,
    rules: Vec<egg::Rewrite<L, N>>,
    node_limit: Option<usize>,
}

pub struct Compiler<
    L: egg::Language + egg::FromOp + Send + Sync + FromPattern + 'static,
    N: egg::Analysis<L> + Default + Clone,
    C: egg::CostFunction<L> + Clone,
> {
    /// All of the rewrite rules for this compiler.
    rules: Vec<egg::Rewrite<L, N>>,

    /// Optional function that filters valid rewrite rules.
    filter: Option<Box<dyn Fn(CostMetric<L, N, C>) -> bool>>,

    /// Phases.
    phases: Vec<Phase<L, N>>,
    cost_fn: C,
    init_node: Option<L>,
    total_node_limit: usize,
    timeout: u64,
    dry_run: bool,
    dump_rules: bool,
    reuse_egraphs: bool,
    rule_distribution: Vec<(PathBuf, Box<dyn Fn(C::Cost) -> f64>)>,
}

impl<L, N, C> Compiler<L, N, C>
where
    L: egg::Language
        + egg::FromOp
        + Send
        + Sync
        + FromPattern
        + std::fmt::Display
        + 'static,
    N: egg::Analysis<L> + Default + Clone,
    C: CostMetrics<L, N> + egg::CostFunction<L> + Clone,
    <C as egg::CostFunction<L>>::Cost: PartialOrd<f64>,
{
    /// Construct a compiler using `cost_fn` to distinguish
    /// between the `source` and `target` languages.
    /// Defaults to using a `node_limit=1_000_000` and
    /// a `timeout=120 seconds`.
    pub fn with_cost_fn(cost_fn: C) -> Self {
        Self {
            rules: vec![],
            phases: vec![],
            filter: None,
            cost_fn,
            init_node: None,
            total_node_limit: 1_000_000,
            timeout: 120,
            dry_run: false,
            dump_rules: false,
            reuse_egraphs: false,
            rule_distribution: vec![],
        }
    }

    /// Return rules read in from a json file.
    pub fn add_external_rules(&mut self, filename: &Path) -> &mut Self {
        self.add_processed_external_rules(filename, |x| x)
    }

    pub fn add_processed_external_rules<F>(
        &mut self,
        filename: &Path,
        proc: F,
    ) -> &mut Self
    where
        F: Fn(egg::Pattern<L>) -> egg::Pattern<L>,
    {
        let contents = std::fs::read_to_string(filename).unwrap();
        let data = json::parse(&contents).unwrap();

        let mut rules = vec![];
        for (idx, eq) in data["eqs"].members().enumerate() {
            let lpat_raw: egg::Pattern<L> =
                eq["lhs"].as_str().unwrap().parse().unwrap();
            let rpat_raw: egg::Pattern<L> =
                eq["rhs"].as_str().unwrap().parse().unwrap();

            let lpat = proc(lpat_raw);
            let rpat = proc(rpat_raw);

            if eq["bidirectional"].as_bool().unwrap() {
                // we have to clone bc it is a bidirectional rule
                rules
                    .extend(rw!(format!("ruler_{}_lr", idx); { lpat.clone() } <=> { rpat.clone() }))
            } else {
                rules
                    .push(rw!(format!("ruler_{}_r", idx); { lpat } => { rpat }))
            }
        }

        self.rules.extend(rules);
        self
    }

    pub fn add_rules(
        &mut self,
        rules: impl Iterator<Item = egg::Rewrite<L, N>>,
    ) -> &mut Self {
        self.rules.extend(rules);
        self
    }

    pub fn with_init_node(&mut self, node: L) -> &mut Self {
        self.init_node = Some(node);
        self
    }

    pub fn with_total_node_limit(&mut self, node_limit: usize) -> &mut Self {
        self.total_node_limit = node_limit;
        self
    }

    pub fn with_timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn with_filter<F: 'static>(&mut self, filter: F) -> &mut Self
    where
        F: Fn(CostMetric<L, N, C>) -> bool,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Define a new phase named `name` using `filter`.
    /// Be careful to call this function after you have added all the rules
    /// to the compiler.
    pub fn add_phase<F: 'static, S>(&mut self, name: S, filter: F) -> &mut Self
    where
        F: Fn(CostMetric<L, N, C>) -> bool,
        S: ToString,
    {
        // gather rules that pass the filter
        let phase_rules = self
            .rules
            .iter()
            .filter(|r| filter(self.cost_fn.all(r)))
            .cloned()
            .collect::<Vec<_>>();

        // construct phase
        self.phases.push(Phase {
            name: name.to_string(),
            rules: phase_rules,
            node_limit: None,
        });

        self
    }

    pub fn dry_run(&mut self) -> &mut Self {
        self.dry_run = true;
        self
    }

    pub fn dump_rules(&mut self) -> &mut Self {
        self.dump_rules = true;
        self
    }

    pub fn output_rule_distribution<P, F>(
        &mut self,
        output_file: P,
        f: F,
    ) -> &mut Self
    where
        P: Into<PathBuf>,
        F: Fn(C::Cost) -> f64 + 'static,
    {
        self.rule_distribution
            .push((output_file.into(), Box::new(f)));
        self
    }

    pub fn reuse_egraphs(&mut self) -> &mut Self {
        self.reuse_egraphs = true;
        self
    }

    /// Load config from a file. This overries anything previously set.
    pub fn with_config(
        &mut self,
        config: &config::CompilerConfiguration,
    ) -> &mut Self {
        self.total_node_limit = config.total_node_limit;
        self.timeout = config.timeout;
        self.dry_run = config.dry_run;
        self.dump_rules = config.dump_rules;
        self.reuse_egraphs = config.reuse_egraphs;
        if let Some(cutoff) = &config.cd_filter {
            let c = cutoff.clone();
            self.with_filter(move |cm| cm.cd > c);
        }
        for phase_config in config.phases.clone() {
            // if disabled, skip this phase
            if let Some(true) = phase_config.disabled {
                continue;
            }

            self.add_phase(phase_config.name, move |cm| {
                let [cd_low, cd_high] = phase_config.cd;
                let [ca_low, ca_high] = phase_config.ca;

                // check conditions. if condition doesn't exist,
                // then default to true
                //   cd conditions
                cd_low.map(|l| cm.cd > l).unwrap_or(true)
                    && cd_high.map(|h| cm.cd <= h).unwrap_or(true)
		    // check ca conditions
                    && ca_low.map(|l| cm.ca > l).unwrap_or(true)
                    && ca_high.map(|h| cm.ca <= h).unwrap_or(true)
            });
        }
        self
    }

    // fn split_rules(&self) -> Vec<(&str, Vec<egg::Rewrite<L, N>>)> {
    //     // first filter all the rules with self.filter
    //     let mut cost_fn = self.cost_fn.clone();
    //     let mut rules = self.rules.clone();
    //     if let Some(f) = &self.filter {
    //         rules.retain(|r| f(cost_fn.all(r)));
    //     }

    //     // then split into phases
    //     if self.phases.is_empty() {
    //         vec![("all rules", self.rules.clone())]
    //     } else {
    //         self.phases
    //             .iter()
    //             .map(|phase| {
    //                 let rules = rules
    //                     .iter()
    //                     .filter(|r| (phase.filter)(cost_fn.all(r)))
    //                     .cloned()
    //                     .collect::<Vec<_>>();
    //                 (phase.name.as_str(), rules)
    //             })
    //             .collect::<Vec<_>>()
    //     }
    // }

    fn equality_saturate(
        &self,
        rules: &[egg::Rewrite<L, N>],
        egraph: egg::EGraph<L, N>,
        prog: &egg::RecExpr<L>,
        node_limit: usize,
    ) -> (C::Cost, egg::RecExpr<L>, egg::EGraph<L, N>) {
        debug!("Making runner");
        let mut runner: egg::Runner<L, N, ()> =
            egg::Runner::new(Default::default())
                .with_egraph(egraph)
                .with_expr(prog)
                .with_node_limit(node_limit)
                .with_time_limit(std::time::Duration::from_secs(self.timeout));

        // TODO make scheduler an option
        // runner = runner.with_scheduler(egg::BackoffScheduler::default());
        runner = runner.with_scheduler(egg::SimpleScheduler);

        // let mut cost_fn = self.cost_fn.clone();
        // for r in rules {
        //     eprintln!(
        //         "[{:?} {:?}] {} <=> {} ({})",
        //         cost_fn.cost_differential(r),
        //         cost_fn.cost_average(r),
        //         r.searcher.get_pattern_ast().unwrap(),
        //         r.applier.get_pattern_ast().unwrap(),
        //         r.name
        //     );
        // }

        runner = runner.run(rules);

        eprintln!("Egraph size: {}", runner.egraph.total_size());
        self.report(&runner);

        // extract the best program
        let extractor =
            egg::Extractor::new(&runner.egraph, self.cost_fn.clone());
        let (cost, prog) = extractor.find_best(runner.roots[0]);
        (cost, prog, runner.egraph)
    }

    pub fn compile(
        &self,
        prog: &egg::RecExpr<L>,
    ) -> (Option<C::Cost>, egg::RecExpr<L>, egg::EGraph<L, N>) {
        self.rule_distribution.iter().for_each(|(path, conv)| {
            self.generate_rule_histogram(path, conv);
        });

        let mut egraph = self.new_egraph();
        let mut cost_fn = self.cost_fn.clone();
        let mut prog = prog.clone();
        let mut cost = None;
        for phase in &self.phases {
            let msg = format!("Starting Phase {}", &phase.name);
            info!("{}", "=".repeat(msg.len()));
            info!("{msg}");
            info!("Using {} rules", phase.rules.len());
            info!("{}", "=".repeat(msg.len()));

            if self.dump_rules {
                for r in &phase.rules {
                    debug!(
                        "[cd:{:.2?} ca:{:.2?}] {} => {} ({})",
                        cost_fn.cost_differential(r),
                        cost_fn.cost_average(r),
                        r.searcher.get_pattern_ast().unwrap(),
                        r.applier.get_pattern_ast().unwrap(),
                        r.name
                    );
                }
            }

            if self.dry_run {
                continue;
            }

            let node_limit = phase.node_limit.unwrap_or(self.total_node_limit);
            let (new_cost, new_prog, new_egraph) =
                self.equality_saturate(&phase.rules, egraph, &prog, node_limit);

            if let Some(old_cost) = cost {
                info!(
                    "Cost: {:?} (old: {:?})",
                    new_cost.clone(),
                    // (self.subtract)(old_cost, new_cost.clone())
                    old_cost
                );
            } else {
                info!("Cost: {:?}", new_cost);
            }

            // update state for next iteration
            (cost, prog) = (Some(new_cost), new_prog);

            // update egraph
            if self.reuse_egraphs {
                egraph = new_egraph;
            } else {
                egraph = self.new_egraph();
            }
        }

        (cost, prog, egraph)
    }

    fn new_egraph(&self) -> egg::EGraph<L, N> {
        let mut egraph =
            egg::EGraph::new(N::default()).with_explanations_disabled();
        if let Some(node) = &self.init_node {
            egraph.add(node.clone());
        }
        egraph
    }

    fn report(&self, runner: &egg::Runner<L, N, ()>) {
        let search_time: f64 =
            runner.iterations.iter().map(|i| i.search_time).sum();
        let apply_time: f64 =
            runner.iterations.iter().map(|i| i.apply_time).sum();
        let rebuild_time: f64 =
            runner.iterations.iter().map(|i| i.rebuild_time).sum();
        let total_time: f64 =
            runner.iterations.iter().map(|i| i.total_time).sum();

        let iters = runner.iterations.len();
        let rebuilds: usize =
            runner.iterations.iter().map(|i| i.n_rebuilds).sum();

        let eg = &runner.egraph;
        eprintln!("  Runner report");
        eprintln!("  =============");
        eprintln!(
            "    Stop reason: {:?}",
            runner.stop_reason.as_ref().unwrap()
        );
        eprintln!("    Iterations: {}", iters);
        eprintln!(
            "    Egraph size: {} nodes, {} classes, {} memo",
            eg.total_number_of_nodes(),
            eg.number_of_classes(),
            eg.total_size()
        );
        eprintln!(
            "    Rebuilds: {}, {:.2} per iter",
            rebuilds,
            (rebuilds as f64) / (iters as f64)
        );
        eprintln!("    Total time: {}", total_time);
        eprintln!(
            "      Search:  ({:.2}) {}",
            search_time / total_time,
            search_time
        );
        eprintln!(
            "      Apply:   ({:.2}) {}",
            apply_time / total_time,
            apply_time
        );
        eprintln!(
            "      Rebuild: ({:.2}) {}",
            rebuild_time / total_time,
            rebuild_time
        );
    }

    fn generate_rule_histogram<F>(&self, path: &PathBuf, f: F)
    where
        F: Fn(C::Cost) -> f64,
    {
        let mut cost_fn = self.cost_fn.clone();
        let avg_data: Vec<(String, f64)> = self
            .rules
            .iter()
            .map(|r| (r.name.to_string(), f(cost_fn.cost_average(r))))
            .collect();
        let diff_data: Vec<(String, f64)> = self
            .rules
            .iter()
            .map(|r| (r.name.to_string(), f(cost_fn.cost_differential(r))))
            .collect();

        let mut file = File::create(path).unwrap();

        writeln!(file, "rule,name,value").unwrap();

        for (r, d) in avg_data {
            writeln!(file, "{r},average,{d}").unwrap();
        }

        for (r, d) in diff_data {
            writeln!(file, "{r},differential,{d}").unwrap();
        }
    }
}

use comp_gen::ruler::{
    self, enumo::*, recipe_utils, ValidationResult,
};
use egg::{self, EGraph, Id};
use log::debug;
use num::integer::Roots;
use rand::Rng;
use rand_pcg::Pcg32;
// ruler no longer has Equality and Synthesizer
use ruler::{
    map, self_product, CVec, SynthAnalysis, SynthLanguage
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf};

use crate::{lang, smt::SmtEquals, Res};

pub fn split_into_halves(n: usize) -> (usize, usize) {
    if n % 2 == 0 {
        (n / 2, n / 2)
    } else {
        (n / 2, n / 2 + 1)
    }
}

/// Perform the integer divison `a / b`.
/// This is defined to be the `n`
/// such that `b * n = a`.
/// If no such `n` exists, then return None.
/// This is defined when b != 0 and there
/// exists some n such that b * n = a.
fn integer_division(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None
    } else if a == 0 {
        Some(0)
    } else {
        Some(a / b)
    }
    // if a / b != 0
}

impl lang::Value {
    fn int1<F>(arg: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(i64) -> lang::Value,
    {
        if let lang::Value::Int(val) = arg {
            Some(f(*val))
        } else {
            None
        }
    }

    fn int2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(i64, i64) -> lang::Value,
    {
        if let (lang::Value::Int(lv), lang::Value::Int(rv)) = (lhs, rhs) {
            Some(f(*lv, *rv))
        } else {
            None
        }
    }

    fn bool2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(bool, bool) -> lang::Value,
    {
        if let (lang::Value::Bool(lv), lang::Value::Bool(rv)) = (lhs, rhs) {
            Some(f(*lv, *rv))
        } else {
            None
        }
    }

    fn vec1<F>(val: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(&[lang::Value]) -> Option<lang::Value>,
    {
        if let lang::Value::Vec(v) = val {
            f(v)
        } else {
            None
        }
    }

    fn vec2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(&[lang::Value], &[lang::Value]) -> Option<lang::Value>,
    {
        if let (lang::Value::Vec(v1), lang::Value::Vec(v2)) = (lhs, rhs) {
            if v1.len() == v2.len() {
                f(v1, v2)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn vec3<F>(v1: &Self, v2: &Self, v3: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(
            &[lang::Value],
            &[lang::Value],
            &[lang::Value],
        ) -> Option<lang::Value>,
    {
        if let (
            lang::Value::Vec(v1),
            lang::Value::Vec(v2),
            lang::Value::Vec(v3),
        ) = (v1, v2, v3)
        {
            if v1.len() == v2.len() && v2.len() == v3.len() {
                f(v1, v2, v3)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn vec2_op<F>(lhs: &Self, rhs: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(&lang::Value, &lang::Value) -> Option<lang::Value>,
    {
        Self::vec2(lhs, rhs, |lhs, rhs| {
            lhs.iter()
                .zip(rhs)
                .map(|(l, r)| f(l, r))
                .collect::<Option<Vec<lang::Value>>>()
                .map(lang::Value::Vec)
        })
    }

    #[allow(unused)]
    fn int_range(min: i64, max: i64, num_samples: usize) -> Vec<lang::Value> {
        (min..=max)
            .step_by(((max - min) as usize) / num_samples)
            .map(lang::Value::Int)
            .collect::<Vec<_>>()
    }

    fn sample_int(
        rng: &mut Pcg32,
        min: i64,
        max: i64,
        num_samples: usize,
    ) -> Vec<lang::Value> {
        (0..num_samples)
            .map(|_| lang::Value::Int(rng.gen_range(min, max)))
            .collect::<Vec<_>>()
    }

    pub fn sample_vec(
        rng: &mut Pcg32,
        min: i64,
        max: i64,
        list_size: usize,
        num_samples: usize,
    ) -> Vec<lang::Value> {
        (0..num_samples)
            .map(|_| {
                lang::Value::Vec(lang::Value::sample_int(
                    rng, min, max, list_size,
                ))
            })
            .collect::<Vec<_>>()
    }
}

fn sgn(x: i64) -> i64 {
    match x.cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn sqrt_sgn(x: i64, y: i64) -> Option<i64> {
    if x >= 0 {
        Some(x.sqrt() * -sgn(y))
    } else {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiosConstant {
    pub kind: String,
    pub value: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiosSeedRules {
    pub lhs: String,
    pub rhs: String,
}

/// Dios configuration struct
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiosConfig {
    pub constants: Vec<DiosConstant>,
    pub seed_rules: Vec<String>,
    pub unops: Vec<String>,
    pub binops: Vec<String>,
    pub triops: Vec<String>,
    pub use_scalar: bool,
    pub use_vector: bool,
    pub variable_duplication: bool,
    pub vector_size: usize,
    pub always_smt: bool,
    pub smt_fallback: bool,
    // pub ruler_config: ruler::SynthParams,
}

impl Default for DiosConfig {
    fn default() -> Self {
        Self {
            constants: vec![],
            seed_rules: vec![],
            unops: vec![],
            binops: vec![],
            triops: vec![],
            use_scalar: false,
            use_vector: false,
            variable_duplication: false,
            vector_size: 1,
            always_smt: false,
            smt_fallback: true,
            // ruler_config: ruler::SynthParams::default(),
        }
    }
}

// Better post-processing support
// TODO JB: I'm sure there's a better way to do this. This is a little bit jank.

/// Reports for each run of Ruler.
// #[derive(Debug, Serialize, Deserialize)]
// #[serde(from = "SerializedRuleset")]
// #[serde(into = "SerializedRuleset")]
// #[serde(bound = "L: SynthLanguage")]



// impl lang::VecLang {
    
//     fn post_process(
//         ruleset: &RuleSet<lang::VecLang>,
//     ) -> Report<Self> {
//         let mut new_eqs: Vec<Equality<_>> = vec![];
//         for eq in &report.eqs {
//             let lhs_pre: Lang = eq.lhs.unpattern().into();
//             let lhs: egg::RecExpr<lang::VecLang> =
//                 lhs_pre.desugar(params.vector_size).into();
//             let rhs_pre: Lang = eq.rhs.unpattern().into();
//             let rhs: egg::RecExpr<lang::VecLang> =
//                 rhs_pre.desugar(params.vector_size).into();
//             if let Some(new_eq) = Equality::new(&lhs, &rhs) {
//                 new_eqs.push(new_eq);
//             } else {
//                 eprintln!("Could not make equation for {} <=> {}", lhs, rhs);
//             }
//         }
//         report.eqs = new_eqs;
//         report
//     }
// }

impl SynthLanguage for lang::VecLang {
    type Constant = lang::Value;
    // type Config = DiosConfig;

    fn to_var(&self) -> Option<egg::Symbol> {
        if let lang::VecLang::Symbol(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    fn mk_var(sym: egg::Symbol) -> Self {
        lang::VecLang::Symbol(sym)
    }

    fn is_constant(&self) -> bool {
        matches!(&self, lang::VecLang::Const(_))
    }

    fn mk_constant(c: <Self as SynthLanguage>::Constant, _e: &mut EGraph<Self, SynthAnalysis>) -> Self {
        lang::VecLang::Const(c)
    }

    fn eval<'a, F>(&'a self, cvec_len: usize, mut get: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self>,
    {
        match self {
            lang::VecLang::Const(i) => vec![Some(i.clone()); cvec_len],
            lang::VecLang::Add([l, r]) => map!(get, l, r => {
                lang::Value::int2(l, r, |l, r| lang::Value::Int(l + r))
            }),
            lang::VecLang::Mul([l, r]) => map!(get, l, r => {
                lang::Value::int2(l, r, |l, r| lang::Value::Int(l * r))
            }),
            lang::VecLang::Minus([l, r]) => map!(get, l, r => {
                lang::Value::int2(l, r, |l, r| lang::Value::Int(l - r))
            }),
            lang::VecLang::Div([l, r]) => get(l)
                .iter()
                .zip(get(r).iter())
                .map(|tup| match tup {
                    (Some(lang::Value::Int(a)), Some(lang::Value::Int(b))) => {
                        integer_division(*a, *b).map(lang::Value::Int)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            lang::VecLang::Or([l, r]) => {
                map!(get, l, r => lang::Value::bool2(l, r, |l, r| lang::Value::Bool(l || r)))
            }
            lang::VecLang::And([l, r]) => {
                map!(get, l, r => lang::Value::bool2(l, r, |l, r| lang::Value::Bool(l && r)))
            }
            lang::VecLang::Ite([_b, _t, _f]) => todo!(),
            lang::VecLang::Lt([l, r]) => {
                map!(get, l, r => lang::Value::int2(l, r, |l, r| lang::Value::Bool(l < r)))
            }
            lang::VecLang::SqrtSgn([l, r]) => map!(
                get,
                l, r => if let (lang::Value::Int(lv), lang::Value::Int(rv)) = (l, r) {
                    sqrt_sgn(*lv, *rv).map(lang::Value::Int)
                } else {
                    None
                }
            ),
            lang::VecLang::Sgn([x]) => {
                map!(get, x => lang::Value::int1(x, |x| lang::Value::Int(sgn(x))))
            }
            lang::VecLang::Sqrt([x]) => get(x)
                .iter()
                .map(|a| match a {
                    Some(lang::Value::Int(a)) => {
                        if *a >= 0 {
                            Some(lang::Value::Int(a.sqrt()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            lang::VecLang::Neg([x]) => {
                map!(get, x => lang::Value::int1(x, |x| lang::Value::Int(-x)))
            }
            lang::VecLang::List(l) => l
                .iter()
                .fold(vec![Some(vec![]); cvec_len], |mut acc, item| {
                    acc.iter_mut().zip(get(item)).for_each(|(mut v, i)| {
                        if let (Some(v), Some(i)) = (&mut v, i) {
                            v.push(i.clone());
                        } else {
                            *v = None;
                        }
                    });
                    acc
                })
                .into_iter()
                .map(|acc| acc.map(lang::Value::List))
                .collect::<Vec<_>>(),
            lang::VecLang::Vec(l) => l
                .iter()
                .fold(vec![Some(vec![]); cvec_len], |mut acc, item| {
                    acc.iter_mut().zip(get(item)).for_each(|(mut v, i)| {
                        if let (Some(v), Some(i)) = (&mut v, i) {
                            v.push(i.clone());
                        } else {
                            *v = None;
                        }
                    });
                    acc
                })
                .into_iter()
                .map(|acc| acc.map(lang::Value::Vec))
                .collect::<Vec<_>>(),
            lang::VecLang::LitVec(l) => l
                .iter()
                .fold(vec![Some(vec![]); cvec_len], |mut acc, item| {
                    acc.iter_mut().zip(get(item)).for_each(|(mut v, i)| {
                        if let (Some(v), Some(i)) = (&mut v, i) {
                            v.push(i.clone());
                        } else {
                            *v = None;
                        }
                    });
                    acc
                })
                .into_iter()
                .map(|acc| acc.map(lang::Value::Vec))
                .collect::<Vec<_>>(),
            #[rustfmt::skip]
            lang::VecLang::Get([l, i]) => map!(get, l, i => {
                if let (lang::Value::Vec(v), lang::Value::Int(idx)) = (l, i) {
                    // get index and clone the inner lang::Value if there is one
                    v.get(*idx as usize).cloned()
                } else {
                    None
                }
            }),
            #[rustfmt::skip]
            lang::VecLang::Concat([l, r]) => map!(get, l, r => {
                lang::Value::vec2(l, r, |l, r| {
                    Some(lang::Value::List(
                        l.iter().chain(r).cloned().collect::<Vec<_>>(),
                    ))
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecAdd([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| {
                    lang::Value::int2(l, r, |l, r| lang::Value::Int(l + r))
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecMinus([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| {
                    lang::Value::int2(l, r, |l, r| lang::Value::Int(l - r))
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecMul([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| {
                    lang::Value::int2(l, r, |l, r| lang::Value::Int(l * r))
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecDiv([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| match (l, r) {
                    (lang::Value::Int(a), lang::Value::Int(b)) => {
                        integer_division(*a, *b).map(lang::Value::Int)
                    }
                    _ => None,
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecMulSgn([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| {
                    lang::Value::int2(l, r, |l, r| lang::Value::Int(sgn(l) * r))
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecSqrtSgn([l, r]) => map!(get, l, r => {
                lang::Value::vec2_op(l, r, |l, r| {
                    if let (lang::Value::Int(lv), lang::Value::Int(rv)) = (l, r) {
                        sqrt_sgn(*lv, *rv).map(lang::Value::Int)
                    } else {
                        None
                    }
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecNeg([l]) => map!(get, l => {
                lang::Value::vec1(l, |l| {
                    if l.iter().all(|x| matches!(x, lang::Value::Int(_))) {
                        Some(lang::Value::Vec(
                            l.iter()
                                .map(|tup| match tup {
                                    lang::Value::Int(a) => lang::Value::Int(-a),
                                    x => panic!("NEG: Ill-formed: {}", x),
                                })
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        None
                    }
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecSqrt([l]) => map!(get, l => {
                lang::Value::vec1(l, |l| {
                    if l.iter().all(|x| {
                        if let lang::Value::Int(i) = x {
                            *i >= 0
                        } else {
                            false
                        }
                    }) {
                        Some(lang::Value::Vec(
                            l.iter()
                                .map(|tup| match tup {
                                    lang::Value::Int(a) => lang::Value::Int(a.sqrt()),
                                    x => panic!("SQRT: Ill-formed: {}", x),
                                })
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        None
                    }
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecSgn([l]) => map!(get, l => {
                lang::Value::vec1(l, |l| {
                    if l.iter().all(|x| matches!(x, lang::Value::Int(_))) {
                        Some(lang::Value::Vec(
                            l.iter()
                                .map(|tup| match tup {
                                    lang::Value::Int(a) => lang::Value::Int(sgn(*a)),
                                    x => panic!("SGN: Ill-formed: {}", x),
                                })
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        None
                    }
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecMAC([acc, v1, v2]) => map!(get, v1, v2, acc => {
                lang::Value::vec3(v1, v2, acc, |v1, v2, acc| {
                    v1.iter()
                        .zip(v2.iter())
                        .zip(acc.iter())
                        .map(|tup| match tup {
                            ((lang::Value::Int(v1), lang::Value::Int(v2)), lang::Value::Int(acc))
				=> Some(lang::Value::Int((v1 * v2) + acc)),
                            _ => None,
                        })
                        .collect::<Option<Vec<lang::Value>>>()
                        .map(lang::Value::Vec)
                })
            }),
            #[rustfmt::skip]
            lang::VecLang::VecMULS([acc, v1, v2]) => map!(get, v1, v2, acc => {
                lang::Value::vec3(v1, v2, acc, |v1, v2, acc| {
                    v1.iter()
                        .zip(v2.iter())
                        .zip(acc.iter())
                        .map(|tup| match tup {
                            ((lang::Value::Int(v1), lang::Value::Int(v2)), lang::Value::Int(acc))
				=> Some(lang::Value::Int(acc - (v1 * v2))),
                            _ => None,
                        })
                        .collect::<Option<Vec<lang::Value>>>()
                        .map(lang::Value::Vec)
                })
            }),
            lang::VecLang::Symbol(_) => vec![],
        }
    }

    fn initialize_vars(egraph: &mut EGraph<Self, SynthAnalysis>, vars: &[String]) {
        *egraph = egraph.clone().with_explanations_enabled();
        debug!("initializing variables");

        // TODO: consts are set upon creating DiosLang config, for now I've hard coded the constants but can clean it up later
        let consts = vec![0,1];
        // let consts = synth
        //     .lang_config
        //     .constants
        //     .iter()
        //     .map(|c| match c.kind.as_str() {
        //         "int" => lang::Value::Int(c.value),
        //         t => todo!("haven't implemented {t} yet."),
        //     })
        //     .collect_vec();

        // when we don't have any constants, just use the number of variables
        // as the cvec size.
        let cvec_size = if consts.is_empty() {
            vars.len()
        } else {
            self_product(
                &consts.iter().map(|x| Some(x.clone())).collect::<Vec<_>>(),
                vars.len(),
            )
            .len()
        } * 5;

        egraph.analysis.cvec_len = cvec_size;

        egraph.analysis.cvec_len = cvec_size;


        // read and add seed rules from config
        // JB: seed rules can be added by means of enumo's extend operator, so there is no longer any reason to add it here :)


        // add constants to egraph
        for v in consts {
            // int part is also hard-coded, fix this!
            egraph.add(lang::VecLang::Const(lang::Value::Int(v)));
        }

        use rand_pcg::Lcg64Xsh32;
        let mut rng = Lcg64Xsh32::new(0,0);

        // add variables
        // set the initial cvecs of variables. this represents all the possible
        // values that this variable can have
        for i in vars {
            let var = egg::Symbol::from(i);
            let id = egraph.add(lang::VecLang::Symbol(var));

            // make the cvec use real data
            let mut cvec = vec![];

            let (n_ints, n_vecs) = split_into_halves(cvec_size);

            cvec.extend(
                lang::Value::sample_int(&mut rng, -100, 100, n_ints)
                    .into_iter()
                    .map(Some),
            );

            cvec.extend(
                lang::Value::sample_vec(
                    &mut rng,
                    -100,
                    100,
                    1, // synth.params.vector_size
                    n_vecs,
                )
                .into_iter()
                .map(Some),
            );

            log::debug!("cvec for `{var}`: {cvec:?}");

            egraph[id].data.cvec = cvec;
        }

    }

    fn validate(
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> ValidationResult {
        // JB TODO: allow for fuzz mode as well
        // let x = if synth.lang_config.always_smt {
            if Self::smt_equals(lhs, rhs) {
                ValidationResult::Valid
            } else {
                ValidationResult::Invalid
            }

    }

}


pub fn vecs_eq(lvec: &CVec<lang::VecLang>, rvec: &CVec<lang::VecLang>) -> bool {
    if lvec.iter().all(|x| x.is_none()) && rvec.iter().all(|x| x.is_none()) {
        false
    } else {
        lvec.iter().zip(rvec.iter()).all(|tup| match tup {
            (Some(l), Some(r)) => l == r,
            (None, None) => true,
            _ => false,
        })
    }
}

fn iter_dios(argnum: usize, depth: usize, values: Vec<&str>, variable_names: Vec<&str>, operations: Vec<Vec<String>>) -> Workload {
    recipe_utils::iter_metric(recipe_utils::base_lang(argnum), "EXPR", Metric::Atoms, depth)
    .filter(Filter::Contains("VAR".parse().unwrap()))
    .plug("VAL", &Workload::new(values))
    .plug("VAR", &Workload::new(variable_names))
    .plug("OP1", &Workload::new(operations[0].clone()))
    .plug("OP2", &Workload::new(operations[1].clone()))
    .plug("OP3", &Workload::new(operations[2].clone()))

}

fn extend_rules() -> ruler::enumo::Ruleset<lang::VecLang>{
    Ruleset::new([
        "(VecAdd (Vec ?b) (Vec ?a)) ==> (Vec (+ ?b ?a))", 
        "(VecDiv (Vec ?b) (Vec ?a)) ==> (Vec (/ ?b ?a))"]
    )
}

fn explore_ruleset_at_depth(current_ruleset: Ruleset<lang::VecLang>, 
                depth: usize, filter: bool, run_name: &str, 
                vals: Vec<&str>, vars: Vec<&str>, ops: Vec<Vec<String>>)
                 -> Ruleset<lang::VecLang>
{
    let start = std::time::Instant::now();
    let mut workload: ruler::enumo::Workload = iter_dios(3, depth, vals.clone(), vars.clone(), ops.clone());
    if filter {
        workload = workload.filter(Filter::Contains("Vec".parse().unwrap()));
    }
    let scheduler = Scheduler::Compress(ruler::Limits::synthesis());
    let egraph = scheduler.run(&workload.to_egraph(), &current_ruleset);
    let mut candidates =  Ruleset::fast_cvec_match(&egraph);
    let mut rules = candidates.minimize(current_ruleset.clone(), scheduler).0;
    println!(
        "Done with generating rules of depth {} after {} secs, {} eclasses",
        depth,
        start.elapsed().as_secs(),
        egraph.number_of_classes()
    );
    ruler::logger::log_rules(&rules, Some((format!("candidates/depth{}_ruleset.json", depth)).as_str()), run_name);
    rules.extend(current_ruleset);
    rules
}

pub fn run(
    dios_config: DiosConfig,
    _chkpt_path: Option<PathBuf>,
) -> Res<ruler::enumo::Ruleset<lang::VecLang>>
{
    let run_name = "garbage..?";
    log::info!("running with config: {dios_config:#?}");

    // add all seed rules
    let mut seed_rules : ruler::enumo::Ruleset<lang::VecLang> = ruler::enumo::Ruleset::default();

    for rule in dios_config.seed_rules.into_iter() {
        match ruler::enumo::Rule::from_string(&rule) {
            Ok(r) => {
                seed_rules.add(r.0);
                match r.1 {
                    Some(bidirectional) => seed_rules.add(bidirectional),
                    None => ()
                }
            },
            Err(msg) => panic!("provided a malformed seed rule, {:?}\n{:?}", rule, msg)
        } 
    }

    // extend based off of any rule patterns we would want to learn
    seed_rules.extend(extend_rules());

    // run the synthesizer
    // JB: now, in order to run the synthesizer, we must create a workload -- tbd, need to figure out how to construct a workload
    let vals = ["0", "1"].to_vec();
    let vars = ["a", "b", "c", "d", "e", "f"].to_vec();
    let ops = [dios_config.unops, dios_config.binops, dios_config.triops].to_vec();

    let rules = seed_rules.clone();
    for idx in 3..=4 {
        let rules = explore_ruleset_at_depth((rules.clone()), idx, false, &run_name, vals.clone(), vars.clone(), ops.clone());
    }
    for idx in 5..=6 {
        let rules = explore_ruleset_at_depth((rules.clone()), idx, true, &run_name, vals.clone(), vars.clone(), ops.clone());
    }



    // learn the depth 4 rules?
    
    // let depth4_workload: ruler::enumo::Workload = iter_dios(3, 4, vals.clone(), vars.clone(), ops.clone());
    // let egraph = scheduler.run(&depth4_workload.to_egraph(), &rules);
    // candidates =  Ruleset::fast_cvec_match(&egraph);
    // let mut rulesd4 = candidates.minimize(seed_rules.clone(), scheduler).0;
    // rulesd4.extend(seed_rules.clone());
    // rulesd4.extend(extend_rules());

    // // learn the depth 5 rules?
    // let depth5_workload: ruler::enumo::Workload = iter_dios(3, 5, vals, vars, ops).filter(Filter::Contains("Vec".parse().unwrap()));
    // let egraph = scheduler.run(&depth5_workload.to_egraph(), &rulesd4);
    // candidates =  Ruleset::fast_cvec_match(&egraph);
    // let mut rulesd5 = candidates.minimize(rulesd4, scheduler).0;
    // rulesd5.extend(rulesd4);

    // let depth6_workload: ruler::enumo::Workload = iter_dios(3, 6, vals, vars, ops).filter(Filter::Contains("Vec".parse().unwrap()));
    // let egraph = scheduler.run(&depth6_workload.to_egraph(), &rulesd4);
    // candidates =  Ruleset::fast_cvec_match(&egraph);
    // let mut rulesd6 = candidates.minimize(rulesd5, scheduler).0;
    // rulesd6.extend(rulesd5);

    // let depth7_workload: ruler::enumo::Workload = iter_dios(3, 7, vals, vars, ops).filter(Filter::Contains("Vec".parse().unwrap()));
    // let egraph = scheduler.run(&depth7_workload.to_egraph(), &rulesd4);
    // candidates =  Ruleset::fast_cvec_match(&egraph);
    // let mut rules = candidates.minimize(rulesd6, scheduler).0;
    // rulesd7.extend(rulesd5);

    // let egraph = scheduler.run(egraph, )

    // let egraph = Scheduler::Simple(ruler::Limits::synthesis()).run(&egraph1, &extend_rules());
    // egraph.dot().to_pdf("egraphs/after_adding_vecadd.pdf");
    // ruler::logger::log_rules(&ruler::enumo::Ruleset::extract_candidates(&egraph1, &egraph), Some("candidates/after_adding_vecadd.json"), run_name);
    // println!(
    //     "Done with phase 2 after {} secs, {} eclasses",
    //     start.elapsed().as_secs(),
    //     egraph.number_of_classes()
    // );
    // std::process::exit(0);

    // let mut candixx/x/dates = Ruleset::fast_cvec_match(&egraph);
    // ruler::logger::log_rules(&candidates, Some("candidates/candidate_ruleset_post_cvec_match1.json"), run_name);

    // let rules = candidates.minimize(seed_rules.clone(), scheduler).0;
    ruler::logger::log_rules(&rules, Some("rulesets/ruleset.json"), run_name);

    Ok(rules)
}

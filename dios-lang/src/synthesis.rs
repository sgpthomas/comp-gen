use anyhow::Context;
use comp_gen::ruler;
use egg::{EGraph, Id, Language};
use itertools::Itertools;
use num::integer::Roots;
use rand::Rng;
use rand_pcg::Pcg64;
use ruler::{
    letter, map, self_product, CVec, Equality, SynthAnalysis, SynthLanguage,
    Synthesizer,
};
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::str::FromStr;
use z3::ast::Ast;

use crate::{lang, Res};

fn split_into_halves(n: usize) -> (usize, usize) {
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
fn integer_division(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else if a == 0 {
        Some(0)
    } else if a / b != 0 {
        Some(a / b)
    } else {
        None
    }
}

impl lang::Value {
    fn int1<F>(arg: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(i32) -> lang::Value,
    {
        if let lang::Value::Int(val) = arg {
            Some(f(*val))
        } else {
            None
        }
    }

    fn int2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<lang::Value>
    where
        F: Fn(i32, i32) -> lang::Value,
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
                .map(|v| lang::Value::Vec(v))
        })
    }

    #[allow(unused)]
    fn int_range(min: i32, max: i32, num_samples: usize) -> Vec<lang::Value> {
        (min..=max)
            .step_by(((max - min) as usize) / num_samples)
            .map(|x| lang::Value::Int(x))
            .collect::<Vec<_>>()
    }

    fn sample_int(
        rng: &mut Pcg64,
        min: i32,
        max: i32,
        num_samples: usize,
    ) -> Vec<lang::Value> {
        (0..num_samples)
            .map(|_| lang::Value::Int(rng.gen_range(min, max)))
            .collect::<Vec<_>>()
    }

    fn sample_vec(
        rng: &mut Pcg64,
        min: i32,
        max: i32,
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

fn sgn(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x == 0 {
        0
    } else {
        -1
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiosConstant {
    pub kind: String,
    pub value: i32,
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
    pub seed_rules: Vec<DiosSeedRules>,
    pub unops: Vec<String>,
    pub binops: Vec<String>,
    pub use_scalar: bool,
    pub use_vector: bool,
    pub vector_mac: bool,
    pub variable_duplication: bool,
    pub vector_size: usize,
    pub always_smt: bool,
    pub smt_fallback: bool,
}

impl Default for DiosConfig {
    fn default() -> Self {
        Self {
            constants: vec![],
            seed_rules: vec![],
            unops: vec![],
            binops: vec![],
            use_scalar: false,
            use_vector: false,
            vector_mac: false,
            variable_duplication: false,
            vector_size: 1,
            always_smt: false,
            smt_fallback: true,
        }
    }
}

impl lang::VecLang {
    fn fuzz_equals(
        synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
        _debug: bool,
    ) -> bool {
        // use fuzzing to determine equality

        // let n = synth.params.num_fuzz;
        // let n = 10;
        let mut env = HashMap::default();

        if lhs.vars().sort() != rhs.vars().sort() {
            eprintln!(
                "lhs vars != rhs vars: {:?} != {:?}",
                lhs.vars().sort(),
                rhs.vars().sort()
            );
            return false;
        }

        for var in lhs.vars() {
            env.insert(var, vec![]);
        }

        for var in rhs.vars() {
            env.insert(var, vec![]);
        }

        // eprintln!("env: {env:?}");

        let (_n_ints, n_vecs) = split_into_halves(10);
        // let (n_neg_ints, n_pos_ints) = split_into_halves(n_ints);

        let possibilities = vec![-10, -5, -2, -1, 0, 1, 2, 5, 10];
        // let possibilities = vec![0, 1, 2];
        for perms in possibilities.iter().permutations(env.keys().len()) {
            for (i, cvec) in perms.iter().zip(env.values_mut()) {
                cvec.push(Some(lang::Value::Int(**i)));
            }
        }

        // add some random vectors
        let mut length = 0;
        for cvec in env.values_mut() {
            cvec.extend(
                lang::Value::sample_vec(
                    &mut synth.rng,
                    -100,
                    100,
                    synth.lang_config.vector_size,
                    n_vecs,
                )
                .into_iter()
                .map(Some),
            );
            length = cvec.len();
        }

        let lvec = Self::eval_pattern(lhs, &env, length);
        let rvec = Self::eval_pattern(rhs, &env, length);

        let debug = false;
        if lvec != rvec || debug {
            log::debug!(
                "  env: {:#?}",
                env.into_iter()
                    .map(|(v, env)| (
                        v,
                        env.into_iter()
                            .flat_map(|x| match x {
                                Some(lang::Value::Vec(v)) =>
                                    Some(lang::Value::Vec(v)),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    ))
                    .collect::<Vec<_>>()
            );
            log::debug!("  lhs: {}, rhs: {}", lhs, rhs);
            log::debug!(
                "  lvec: {:#?}, rvec: {:#?}",
                lvec.iter().flatten().collect_vec(),
                rvec.iter().flatten().collect_vec()
            );
            log::debug!("  eq: {}", vecs_eq(&lvec, &rvec));
        }

        vecs_eq(&lvec, &rvec)
    }

    fn smt_equals(
        synth: &mut Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool {
        let mut cfg = z3::Config::new();
        cfg.set_timeout_msec(1000);
        let ctx = z3::Context::new(&cfg);
        let solver = z3::Solver::new(&ctx);

        let left = egg_to_z3(&ctx, &Self::instantiate(lhs));
        let right = egg_to_z3(&ctx, &Self::instantiate(rhs));

        // if we can translate egg to z3 for both lhs, and rhs, then
        // run the z3 solver. otherwise fallback to fuzz_equals
        if let (Some(lexpr), Some(rexpr)) = (&left, &right) {
            log::debug!("z3 check {} != {}", lexpr, rexpr);

            // check to see if lexpr is NOT equal to rexpr.
            // if we can't find a counter example to this
            // then we know that they are equal.
            solver.assert(&lexpr._eq(rexpr).not());

            // let fuzz = Self::fuzz_equals(synth, lhs, rhs, false);
            let smt = match solver.check() {
                z3::SatResult::Unsat => true,
                z3::SatResult::Unknown | z3::SatResult::Sat => false,
            };

            // if smt != fuzz {
            //     log::info!("{lhs} <=> {rhs} Found ya!!!");
            //     let mut cfg = z3::Config::new();
            //     cfg.set_timeout_msec(1000);
            //     let ctx = z3::Context::new(&cfg);
            //     log::info!("smt: {smt}, fuzz: {fuzz}");
            //     log::info!(
            //         "smt formula: {} = {}",
            //         egg_to_z3(&ctx, &Self::instantiate(lhs))
            //             .map(|x| x.to_string())
            //             .unwrap_or("".to_string()),
            //         egg_to_z3(&ctx, &Self::instantiate(rhs))
            //             .map(|x| x.to_string())
            //             .unwrap_or("".to_string())
            //     );
            //     log::info!("{:?}", solver.get_model());
            //     Self::fuzz_equals(synth, lhs, rhs, true);
            // } else {
            //     log::debug!("smt: {smt}, fuzz: {fuzz}")
            // }

            smt
        } else {
            Self::fuzz_equals(synth, lhs, rhs, false)
        }
    }
}

impl SynthLanguage for lang::VecLang {
    type Constant = lang::Value;
    type Config = DiosConfig;

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

    fn to_constant(&self) -> Option<&Self::Constant> {
        if let lang::VecLang::Const(n) = self {
            Some(n)
        } else {
            None
        }
    }

    fn mk_constant(c: <Self as SynthLanguage>::Constant) -> Option<Self> {
        Some(lang::VecLang::Const(c))
    }

    fn eval<'a, F>(&'a self, cvec_len: usize, mut get: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self>,
    {
        match self {
            lang::VecLang::Const(i) => vec![Some(i.clone()); cvec_len],
            lang::VecLang::Add([l, r]) => {
                map!(get, l, r => lang::Value::int2(l, r, |l, r| lang::Value::Int(l + r)))
            }
            lang::VecLang::Mul([l, r]) => {
                map!(get, l, r => lang::Value::int2(l, r, |l, r| lang::Value::Int(l * r)))
            }
            lang::VecLang::Minus([l, r]) => {
                map!(get, l, r => lang::Value::int2(l, r, |l, r| lang::Value::Int(l - r)))
            }
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
                .map(|acc| {
                    if let Some(x) = acc {
                        Some(lang::Value::List(x))
                    } else {
                        None
                    }
                })
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
                .map(|acc| {
                    if let Some(x) = acc {
                        Some(lang::Value::Vec(x))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            // lang::VecLang::LitVec(x) => todo!(),
            #[rustfmt::skip]
            lang::VecLang::Get([l, i]) => map!(get, l, i => {
                if let (lang::Value::Vec(v), lang::Value::Int(idx)) = (l, i) {
                    // get index and clone the inner lang::Value if there is one
                    v.get(*idx as usize).map(|inner| inner.clone())
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
            lang::VecLang::VecMulSgn(_) => todo!(),
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
                        .map(|x| lang::Value::Vec(x))
                })
            }),
            lang::VecLang::Symbol(_) => vec![],
        }
    }

    fn init_synth(synth: &mut Synthesizer<Self, ruler::Uninit>) {
        let consts = synth
            .lang_config
            .constants
            .iter()
            .map(|c| match c.kind.as_str() {
                "int" => lang::Value::Int(c.value),
                t => todo!("haven't implemented {t} yet."),
            })
            .collect_vec();

        // when we don't have any constants, just use the number of variables
        // as the cvec size.
        let cvec_size = if consts.is_empty() {
            synth.params.variables
        } else {
            self_product(
                &consts.iter().map(|x| Some(x.clone())).collect::<Vec<_>>(),
                synth.params.variables,
            )
            .len()
        };

        // read and add seed rules from config
        for rule in &synth.lang_config.seed_rules {
            let rule: Equality<lang::VecLang> = Equality::new(
                &rule.lhs.parse().unwrap(),
                &rule.rhs.parse().unwrap(),
            )
            .unwrap();
            synth
                .equalities
                .insert(format!("{} <=> {}", rule.lhs, rule.rhs).into(), rule);
        }

        let mut egraph = egg::EGraph::new(SynthAnalysis {
            cvec_len: cvec_size,
        });

        // add constants to egraph
        for v in consts {
            egraph.add(lang::VecLang::Const(v));
        }

        // add variables
        for i in 0..synth.params.variables {
            let var = egg::Symbol::from(letter(i));
            let id = egraph.add(lang::VecLang::Symbol(var));

            // make the cvec use real data
            let mut cvec = vec![];

            let (n_ints, n_vecs) = split_into_halves(cvec_size);

            cvec.extend(
                lang::Value::sample_int(&mut synth.rng, -100, 100, n_ints)
                    .into_iter()
                    .map(Some),
            );

            cvec.extend(
                lang::Value::sample_vec(
                    &mut synth.rng,
                    -100,
                    100,
                    1, // synth.params.vector_size
                    n_vecs,
                )
                .into_iter()
                .map(Some),
            );

            egraph[id].data.cvec = cvec;
        }

        // set egraph to the one we just constructed
        synth.egraph = egraph;
    }

    /// Plan for `make_layer`
    /// even iter
    ///   normal binary ops
    /// odd iter
    ///   depth 1 and depth 2 vector operations
    fn make_layer<'a>(
        ids: Vec<Id>,
        synth: &'a Synthesizer<Self, ruler::Init>,
        mut iter: usize,
    ) -> Box<dyn Iterator<Item = Self> + 'a> {
        // vd for variable duplication
        let vd = synth.lang_config.variable_duplication;

        // if iter % 2 == 0 {
        iter = iter - 1; // make iter start at 0

        // only do binops for iters < 2
        let binops = if iter < 2 && synth.lang_config.use_scalar {
            let us = ids
                .clone()
                .into_iter()
                .filter(move |x| !synth.egraph[*x].data.exact)
                .map(move |x| {
                    synth
                        .lang_config
                        .unops
                        .iter()
                        .map(|op| match op.as_str() {
                            "neg" => lang::VecLang::Neg([x]),
                            "sgn" => lang::VecLang::Sgn([x]),
                            "sqrt" => lang::VecLang::Sqrt([x]),
                            _ => panic!("Unknown vec unop"),
                        })
                        .collect_vec()
                })
                .flatten();
            let bs = (0..2)
                .map(|_| ids.clone())
                .multi_cartesian_product()
                .filter(move |ids| {
                    !ids.iter().all(|x| synth.egraph[*x].data.exact)
                })
                .map(|ids| [ids[0], ids[1]])
                .map(move |x| {
                    synth
                        .lang_config
                        .binops
                        .iter()
                        .map(|op| match op.as_str() {
                            "+" => lang::VecLang::Add(x),
                            "*" => lang::VecLang::Mul(x),
                            "-" => lang::VecLang::Minus(x),
                            "/" => lang::VecLang::Div(x),
                            "or" => lang::VecLang::Or(x),
                            "&&" => lang::VecLang::And(x),
                            "<" => lang::VecLang::Lt(x),
                            _ => panic!("Unknown binop"),
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .filter(move |node| vd || unique_vars(node, &synth.egraph));
            Some(us.chain(bs))
        } else {
            None
        };

        let vec_stuff = if synth.lang_config.use_vector {
            let vec_unops = ids
                .clone()
                .into_iter()
                .filter(move |x| !synth.egraph[*x].data.exact)
                .map(move |x| {
                    let mut v = synth
                        .lang_config
                        .unops
                        .iter()
                        .map(|op| match op.as_str() {
                            "neg" => lang::VecLang::VecNeg([x]),
                            "sgn" => lang::VecLang::VecSgn([x]),
                            "sqrt" => lang::VecLang::VecSqrt([x]),
                            _ => panic!("Unknown vec unop"),
                        })
                        .collect_vec();
                    v.extend(vec![lang::VecLang::Vec(
                        vec![x].into_boxed_slice(),
                    )]);
                    v
                })
                .flatten();

            let vec_binops = (0..2)
                .map(|_| ids.clone())
                .multi_cartesian_product()
                .filter(move |ids| {
                    !ids.iter().all(|x| synth.egraph[*x].data.exact)
                })
                .map(|ids| [ids[0], ids[1]])
                .map(move |x| {
                    synth
                        .lang_config
                        .binops
                        .iter()
                        .map(|op| match op.as_str() {
                            "+" => lang::VecLang::VecAdd(x),
                            "-" => lang::VecLang::VecMinus(x),
                            "*" => lang::VecLang::VecMul(x),
                            "/" => lang::VecLang::VecDiv(x),
                            _ => panic!("Unknown vec binop"),
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .filter(move |node| vd || unique_vars(node, &synth.egraph));

            Some(vec_unops.chain(vec_binops))
        } else {
            None
        };

        let vec_mac = if synth.lang_config.vector_mac {
            let vec_mac = (0..3)
                .map(|_| ids.clone())
                .multi_cartesian_product()
                .filter(move |ids| {
                    !ids.iter().all(|x| synth.egraph[*x].data.exact)
                })
                .map(|ids| [ids[0], ids[1], ids[2]])
                .map(move |x| lang::VecLang::VecMAC(x))
                .filter(move |node| vd || unique_vars(node, &synth.egraph));
            Some(vec_mac)
        } else {
            None
        };

        match (binops, vec_stuff, vec_mac) {
            // all are defined
            (Some(b), Some(v), Some(m)) => Box::new(b.chain(v).chain(m)),
            // two are defined
            (Some(i), Some(j), _) => Box::new(i.chain(j)),
            (Some(i), _, Some(j)) => Box::new(i.chain(j)),
            (_, Some(i), Some(j)) => Box::new(i.chain(j)),
            // one is defined
            (Some(i), _, _) => Box::new(i),
            (_, Some(i), _) => Box::new(i),
            (_, _, Some(i)) => Box::new(i),
            // none are defined
            (_, _, _) => panic!(),
        }
    }

    fn is_valid(
        synth: &mut Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool {
        // debug_rule(
        //     synth,
        //     "(VecDiv (VecMAC ?f ?e ?d) (VecMAC ?c ?b ?a))",
        //     "(VecDiv ?d ?f)",
        //     1,
        //     &[
        //         ("?a", Value::Int(51)),
        //         ("?b", Value::Int(-15)),
        //         ("?c", Value::Int(88)),
        //         ("?d", Value::Int(22)),
        //         ("?e", Value::Int(79)),
        //         ("?f", Value::Int(-81)),
        //     ],
        // );
        // panic!();

        if synth.lang_config.always_smt {
            Self::smt_equals(synth, lhs, rhs)
        } else {
            let fuzz = Self::fuzz_equals(synth, lhs, rhs, false);
            // if fuzz fails and `smt_fallback` is enabled, run
            // `smt_equals`.
            if synth.lang_config.smt_fallback && !fuzz {
                Self::smt_equals(synth, lhs, rhs)
            } else {
                fuzz
            }
        }
    }

    // fn post_process(
    //     params: &SynthParams,
    //     mut report: ruler::Report<Self>,
    // ) -> ruler::Report<Self> {
    //     let mut new_eqs: Vec<Equality<_>> = vec![];
    //     for eq in &report.eqs {
    //         let lhs_pre: Lang = eq.lhs.unpattern().into();
    //         let lhs: egg::RecExpr<lang::VecLang> =
    //             lhs_pre.desugar(params.vector_size).into();
    //         let rhs_pre: Lang = eq.rhs.unpattern().into();
    //         let rhs: egg::RecExpr<lang::VecLang> =
    //             rhs_pre.desugar(params.vector_size).into();
    //         if let Some(new_eq) = Equality::new(&lhs, &rhs) {
    //             new_eqs.push(new_eq);
    //         } else {
    //             eprintln!("Could not make equation for {} <=> {}", lhs, rhs);
    //         }
    //     }
    //     report.eqs = new_eqs;
    //     report
    // }
}

fn get_vars(
    node: &lang::VecLang,
    egraph: &EGraph<lang::VecLang, SynthAnalysis>,
) -> Vec<egg::Symbol> {
    node.fold(vec![], |mut acc, id| {
        let node = &egraph[id].nodes[0];
        if node.is_leaf() {
            acc.extend(egraph[id].data.vars.iter());
        } else {
            acc.extend(get_vars(node, egraph));
        }
        acc
    })
}

fn unique_vars(
    node: &lang::VecLang,
    egraph: &EGraph<lang::VecLang, SynthAnalysis>,
) -> bool {
    let vars: Vec<egg::Symbol> = get_vars(node, egraph);
    vars.iter().all_unique()
}

fn vecs_eq(lvec: &CVec<lang::VecLang>, rvec: &CVec<lang::VecLang>) -> bool {
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

#[allow(unused)]
fn debug_rule(
    synth: &mut Synthesizer<lang::VecLang, ruler::Init>,
    left: &str,
    right: &str,
    n: usize,
    env_pairs: &[(&str, lang::Value)],
) {
    let mut env: HashMap<
        egg::Var,
        Vec<Option<lang::Value>>,
        BuildHasherDefault<FxHasher>,
    > = HashMap::default();

    env_pairs.iter().for_each(|(var, value)| {
        env.insert(egg::Var::from_str(var).unwrap(), vec![Some(value.clone())]);
    });

    let pleft: egg::Pattern<lang::VecLang> = left.parse().unwrap();
    let pright: egg::Pattern<lang::VecLang> = right.parse().unwrap();
    log::info!("left: {pleft}");
    let lres = lang::VecLang::eval_pattern(&pleft, &env, n);
    log::info!("right: {pright}");
    let rres = lang::VecLang::eval_pattern(&pright, &env, n);
    log::info!(
        "TEST:\n  {lres:?}\n    ?= ({})\n  {rres:?}",
        vecs_eq(&lres, &rres),
    );
    log::info!("{} => {}", left, right);

    // try fuzzing
    lang::VecLang::smt_equals(synth, &pleft, &pright);
}

pub fn run(
    params: ruler::SynthParams,
    opts: crate::SynthOpts,
) -> Res<ruler::Report<lang::VecLang>> {
    let dios_config = if let Some(config) = opts.config {
        let config_file =
            File::open(config.clone()).context("open config path")?;
        let parsed: DiosConfig = serde_json::from_reader(config_file)
            .context(format!("parse {config:?} as dios_config"))?;
        parsed
    } else {
        DiosConfig::default()
    };

    log::info!("running with config: {dios_config:#?}");
    log::info!("running with synth config {params:#?}");

    // create the synthesizer
    let syn = ruler::Synthesizer::<lang::VecLang, _>::new_with_data(
        params.clone(),
        dios_config,
    )
    .init();
    // run the synthesizer
    let report = syn.run();

    Ok(lang::VecLang::post_process(&params, report))
}

/// Translate an egg::RecExpr into an equivalent z3 expression.
/// This only works for operations on integers for now.
fn egg_to_z3<'a>(
    ctx: &'a z3::Context,
    expr: &egg::RecExpr<lang::VecLang>,
) -> Option<z3::ast::Int<'a>> {
    // This translate works by walking through the RecExpr vector
    // in order, and translating just that node. We push this
    // translation into a buffer as we go. Any time we need to
    // reference a child, we can look up the corresponding index
    // in the buffer. This works because we have the guarantee that
    // some element i in RecExpr only ever refers to elements j < i.
    // By the time we need them, we will have already translated them.

    let mut buf: Vec<z3::ast::Int> = vec![];

    for node in expr.as_ref().iter() {
        match node {
            lang::VecLang::Const(lang::Value::Int(i)) => {
                buf.push(z3::ast::Int::from_i64(&ctx, *i as i64))
            }
            lang::VecLang::Symbol(v) => {
                buf.push(z3::ast::Int::new_const(ctx, v.to_string()))
            }
            lang::VecLang::Add([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int + y_int);
            }
            lang::VecLang::Mul([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int * y_int);
            }
            lang::VecLang::Minus([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int - y_int);
            }
            lang::VecLang::Div([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int / y_int);
            }
            lang::VecLang::Neg([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(-x_int);
            }

            // an attempt to support vector operators.
            // I think I should just be able to map them
            // on to their scalar counter parts.
            lang::VecLang::VecAdd([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int + y_int);
            }
            lang::VecLang::VecMinus([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int - y_int);
            }
            lang::VecLang::VecMul([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int * y_int);
            }
            lang::VecLang::VecDiv([x, y]) => {
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push(x_int / y_int);
            }
            lang::VecLang::VecNeg([x]) => {
                let x_int = &buf[usize::from(*x)];
                buf.push(-x_int);
            }
            lang::VecLang::VecMAC([acc, x, y]) => {
                let acc_int = &buf[usize::from(*acc)];
                let x_int = &buf[usize::from(*x)];
                let y_int = &buf[usize::from(*y)];
                buf.push((x_int * y_int) + acc_int);
            }

            // unsupported operations
            // return early
            lang::VecLang::Const(_)
            | lang::VecLang::Or(_)
            | lang::VecLang::And(_)
            | lang::VecLang::Ite(_)
            | lang::VecLang::Lt(_)
            | lang::VecLang::Sgn(_)
            | lang::VecLang::Sqrt(_)
            | lang::VecLang::List(_)
            | lang::VecLang::Vec(_)
            | lang::VecLang::Get(_)
            | lang::VecLang::Concat(_)
            | lang::VecLang::VecMulSgn(_)
            | lang::VecLang::VecSqrt(_)
            | lang::VecLang::VecSgn(_) => return None,
        }
    }
    // return the last element
    buf.pop()
}

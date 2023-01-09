use comp_gen::ruler::{
    self,
    egg::{self, EGraph, Id, Language},
};
use itertools::Itertools;
use log::debug;
use num::integer::Roots;
use rand::Rng;
use rand_pcg::Pcg32;
use ruler::{
    letter, map, self_product, CVec, Equality, SynthAnalysis, SynthLanguage,
    Synthesizer,
};
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::hash::BuildHasherDefault;
use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};

use crate::{fuzz::FuzzEquals, lang, smt::SmtEquals, Res};

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
    pub ruler_config: ruler::SynthParams,
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
            ruler_config: ruler::SynthParams::default(),
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
                        .map(lang::Value::Vec)
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
        } * 10;

        debug!("cvec size: {cvec_size}");

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

        ////// HACK
        // lol i don't remember why this is a hack. I think I wanted this to be done
        // inside of ruler?
        if synth.params.enable_explanations {
            egraph = egraph.with_explanations_enabled();
        }
        ////// HACK

        // add constants to egraph
        for v in consts {
            egraph.add(lang::VecLang::Const(v));
        }

        // add variables
        // set the initial cvecs of variables. this represents all the possible
        // values that this variable can have
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

            log::debug!("cvec for `{var}`: {cvec:?}");

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
        iter -= 1; // make iter start at 0

        // only do binops for iters < 2
        let binops = if iter < 2 && synth.lang_config.use_scalar {
            let us = ids
                .clone()
                .into_iter()
                .filter(move |x| !synth.egraph[*x].data.exact)
                .flat_map(move |x| {
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
                });
            let bs = (0..2)
                .map(|_| ids.clone())
                .multi_cartesian_product()
                .filter(move |ids| {
                    !ids.iter().all(|x| synth.egraph[*x].data.exact)
                })
                .map(|ids| [ids[0], ids[1]])
                .flat_map(move |x| {
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
                .flat_map(move |x| {
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
                });

            let vec_binops = (0..2)
                .map(|_| ids.clone())
                .multi_cartesian_product()
                .filter(move |ids| {
                    !ids.iter().all(|x| synth.egraph[*x].data.exact)
                })
                .map(|ids| [ids[0], ids[1]])
                .flat_map(move |x| {
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
                .map(lang::VecLang::VecMAC)
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
            (_, _, _) => panic!("No binops or vector ops defined."),
        }
    }

    fn is_valid(
        synth: &mut Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool {
        let x = if synth.lang_config.always_smt {
            Self::smt_equals(synth, lhs, rhs)
        } else {
            let fuzz = Self::fuzz_equals(synth, lhs, rhs, false);
            // if fuzz succeeds and `smt_fallback` is enabled, run `smt_equals`.
            if synth.lang_config.smt_fallback && fuzz {
                debug!("falling back to smt");
                Self::smt_equals(synth, lhs, rhs)
            } else {
                false
            }
        };
        debug!("Checking {lhs} => {rhs}: {x}");
        x
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

    fn debug_pre_union(
        egraph: &EGraph<Self, SynthAnalysis>,
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
                    let extractor = egg::Extractor::new(egraph, egg::AstDepth);
                    let (_, prog1) = extractor.find_best(id1);
                    let (_, prog2) = extractor.find_best(id2);

                    log::error!("Trying to merge:");
                    log::error!(
                        "  {} <=> {}",
                        prog1.pretty(80),
                        prog2.pretty(80)
                    );
                    log::error!("cvec1: {:?}", egraph[id1].data.cvec);
                    log::error!("cvec2: {:?}", egraph[id2].data.cvec);
                    log::error!("just: {justification:?}");

                    if let Some(egg::Justification::Rule(s)) = &justification {
                        if let Some((lhs, rhs)) =
                            s.as_str().split(" => ").collect_tuple()
                        {
                            // strip off the ? prefix for variable names
                            let lhs_expr: egg::RecExpr<_> = strip_prefix(
                                &lhs.parse::<egg::RecExpr<_>>().unwrap(),
                            );

                            let rhs_expr: egg::RecExpr<_> = strip_prefix(
                                &rhs.parse::<egg::RecExpr<_>>().unwrap(),
                            );

                            log::error!("{lhs_expr:?} <=> {rhs_expr:?}");

                            // print out ids, plus cannoncial ids
                            let lhs_id = egraph
                                .lookup_expr(&lhs_expr)
                                .map(|x| egraph.find(x));
                            let rhs_id = egraph
                                .lookup_expr(&rhs_expr)
                                .map(|x| egraph.find(x));
                            log::error!(
                                "{lhs_id:?} => {rhs_id:?}, {id1}({}) => {id2}({})",
				egraph.find(id1),
				egraph.find(id2),
                            );

                            // let's get the cvec for a
                            let a_id = egraph
                                .lookup_expr(&egg::RecExpr::from(vec![
                                    lang::VecLang::Symbol("a".into()),
                                ]))
                                .unwrap();
                            log::error!(
                                "'a' cvec: {:?}",
                                egraph[a_id].data.cvec
                            );
                        } else {
                            log::error!("Split failed: '{s}'");
                        }
                    } else {
                        log::error!("No rule justification");
                    }

                    panic!("This is going to cause problems. Let's just stop here.");
                }
                _ => (),
            }
        }
    }
}

fn strip_prefix(
    expr: &egg::RecExpr<lang::VecLang>,
) -> egg::RecExpr<lang::VecLang> {
    expr.as_ref()
        .iter()
        .map(|n: &lang::VecLang| match n {
            lang::VecLang::Symbol(s) => lang::VecLang::Symbol(
                s.as_str().strip_prefix("?").unwrap().into(),
            ),
            x => x.clone(),
        })
        .collect_vec()
        .into()
}

fn get_vars(
    node: &lang::VecLang,
    egraph: &EGraph<lang::VecLang, SynthAnalysis>,
    seen: &mut HashMap<egg::Id, Vec<egg::Symbol>>,
) -> Vec<egg::Symbol> {
    node.fold(vec![], |mut acc, id| {
        // if we haven't already seen this id, gather vars from children

        if let Some(cached) = seen.get(&id) {
            // if we have already seen this id, add on the cached results
            acc.extend_from_slice(cached)
        } else {
            // else, get the variables for this node, recursing if necessary
            let node = &egraph[id].nodes[0];
            let vars = if node.is_leaf() {
                egraph[id].data.vars.clone()
            } else {
                get_vars(node, egraph, seen)
            };
            // save the results in seen
            seen.insert(id, vars.clone());
            acc.extend(vars);
        }
        acc
    })
}

fn unique_vars(
    node: &lang::VecLang,
    egraph: &EGraph<lang::VecLang, SynthAnalysis>,
) -> bool {
    let mut seen: HashMap<egg::Id, Vec<egg::Symbol>> = HashMap::default();
    let vars: Vec<egg::Symbol> = get_vars(node, egraph, &mut seen);
    vars.iter().all_unique()
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
    dios_config: DiosConfig,
    chkpt_path: Option<PathBuf>,
) -> Res<ruler::Report<lang::VecLang>> {
    log::info!("running with config: {dios_config:#?}");

    // create the synthesizer
    let mut syn = ruler::Synthesizer::<lang::VecLang, _>::new_with_data(
        dios_config.ruler_config.clone(),
        dios_config.clone(),
    )
    .init();

    if let Some(chkpt) = chkpt_path {
        syn.load_checkpoint(&chkpt);
    }

    // // debug
    // let res = lang::VecLang::smt_equals(
    //     &mut syn,
    //     &"(- (/ ?a ?a) (/ 0 ?a))".parse().unwrap(),
    //     &"(sgn (* ?a ?a))".parse().unwrap(),
    // );
    // debug!("res: {res}");
    // let res = lang::VecLang::smt_equals(
    //     &mut syn,
    //     &"(/ 0 ?a)".parse().unwrap(),
    //     &"0".parse().unwrap(),
    // );
    // debug!("res: {res}");
    // panic!();
    // // debug

    // run the synthesizer
    let report = syn.run();

    Ok(lang::VecLang::post_process(
        &dios_config.ruler_config,
        report,
    ))
}

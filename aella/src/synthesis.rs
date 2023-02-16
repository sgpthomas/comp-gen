use itertools::Itertools;
use rand::Rng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::lang::Aella;
use comp_gen::ruler::{
    self,
    egg::{self, Id},
};

type Env = BTreeMap<egg::Symbol, i64>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PrimVal {
    Int(i64),
    Var(egg::Symbol),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Value(Option<PrimVal>, Env);

impl Value {
    fn prim(prim: PrimVal) -> Self {
        Self(Some(prim), BTreeMap::default())
    }

    fn env(sym: egg::Symbol, val: i64) -> Self {
        Self(None, BTreeMap::from([(sym, val)]))
    }

    fn int(&self) -> Option<&i64> {
        let Value(val, env) = self;

        val.as_ref().and_then(|x| match x {
            PrimVal::Int(i) => Some(i),
            PrimVal::Var(name) => env.get(&name),
        })
    }

    fn val2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<Value>
    where
        F: Fn(i64, i64) -> PrimVal,
    {
        if let (Some(lv), Some(rv)) = (lhs.int(), rhs.int()) {
            Some(Value::prim(f(*lv, *rv)))
        } else {
            None
        }
    }

    fn random_env(rng: &mut Pcg32, prim: usize, n_vars: usize) -> Value {
        let mut map = BTreeMap::default();
        for i in 0..n_vars {
            let rand_sym = egg::Symbol::from(ruler::letter(i));
            map.insert(rand_sym, rng.gen_range(-100, 100));
        }

        let prim = if prim == 0 {
            Some(PrimVal::Int(rng.gen_range(-100, 100)))
        } else if prim == 1 {
            let rand_sym =
                egg::Symbol::from(ruler::letter(rng.gen_range(0, 26)));
            // make sure that the name of this variable is bound in the environment
            map.insert(rand_sym, rng.gen_range(-100, 100));
            Some(PrimVal::Var(rand_sym))
        } else {
            None
        };

        Value(prim, map)
    }
}

fn disjoint_union(mut e1: Env, e2: &Env) -> Option<Env> {
    for (k, v) in e2.into_iter() {
        if e1.contains_key(k) {
            // panic!("Maps overlap at {}: {:?} {:?}", k, e1, e2);
            return None;
        }

        e1.insert(*k, *v);
    }
    Some(e1)
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ruler::SynthLanguage for Aella {
    type Constant = Value;
    type Config = crate::SynthOpts;

    fn eval<'a, F>(&'a self, cvec_len: usize, mut get: F) -> ruler::CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a ruler::CVec<Self>,
    {
        // println!("expr: {self}");
        let x = match self {
            Aella::Plus([l, r]) => {
                ruler::map!(get, l, r => Value::val2(l, r, |l, r| PrimVal::Int(l + r)))
            }
            Aella::Sub([l, r]) => {
                ruler::map!(get, l, r => Value::val2(l, r, |l, r| PrimVal::Int(l - r)))
            }
            Aella::Times([l, r]) => {
                ruler::map!(get, l, r => Value::val2(l, r, |l, r| PrimVal::Int(l * r)))
            }
            Aella::Div([l, r]) => {
                ruler::map!(get, l, r => Value::val2(l, r, |l, r| PrimVal::Int(l / r)))
            }
            Aella::Eq([_l, _r]) => todo!(),
            Aella::Lt([_l, _r]) => todo!(),
            Aella::Not([_inner]) => todo!(),
            Aella::And([_l, _r]) => todo!(),
            Aella::Seq([l, r]) => get(l)
                .iter()
                .zip(get(r).iter())
                .map(|tup| match tup {
                    (Some(Value(None, e1)), Some(Value(None, e2))) => {
                        disjoint_union(e1.clone(), e2).map(|e| Value(None, e))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            Aella::Assign([l, r]) => get(l)
                .iter()
                .zip(get(r).iter())
                .map(|tup| match tup {
                    (Some(Value(Some(PrimVal::Var(name)), _)), Some(val)) => {
                        val.int().map(|val| Value::env(*name, *val))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            Aella::While(_) => todo!(),
            Aella::AsmMov([rd, src]) => get(rd)
                .iter()
                .zip(get(src).iter())
                .map(|tup| match tup {
                    (Some(Value(Some(PrimVal::Var(name)), _)), Some(val)) => {
                        val.int().map(|val| Value::env(*name, *val))
                    }
                    _ => None,
                })
                .collect_vec(),
            Aella::AsmAdd([rd, rn, rm]) => get(rd)
                .iter()
                .zip(get(rn).iter())
                .zip(get(rm).iter())
                .map(|tup| match tup {
                    (
                        (Some(Value(Some(PrimVal::Var(name)), _)), Some(l)),
                        Some(r),
                    ) => l
                        .int()
                        .zip(r.int())
                        .map(|(l, r)| Value::env(*name, l + r)),
                    _ => None,
                })
                .collect_vec(),
            Aella::AsmSub([rd, rn, rm]) => get(rd)
                .iter()
                .zip(get(rn).iter())
                .zip(get(rm).iter())
                .map(|tup| match tup {
                    (
                        (Some(Value(Some(PrimVal::Var(name)), _)), Some(l)),
                        Some(r),
                    ) => l
                        .int()
                        .zip(r.int())
                        .map(|(l, r)| Value::env(*name, l - r)),
                    _ => None,
                })
                .collect_vec(),
            Aella::AsmSmull([rd, rn, rm]) => get(rd)
                .iter()
                .zip(get(rn).iter())
                .zip(get(rm).iter())
                .map(|tup| match tup {
                    (
                        (Some(Value(Some(PrimVal::Var(name)), _)), Some(l)),
                        Some(r),
                    ) => l
                        .int()
                        .zip(r.int())
                        .map(|(l, r)| Value::env(*name, l * r)),
                    _ => None,
                })
                .collect_vec(),
            Aella::AsmSdiv(_) => todo!(),
            Aella::Num(i) => {
                vec![Some(Value::prim(PrimVal::Int(*i))); cvec_len + 2]
            }
            Aella::Var(_x) => {
                // I'm not sure why this is just empty tbh.
                // my guess is that it's because we never
                // actually evaluate this. it's cvec comes
                // from initialization. still strange.
                // unreachable!("I guess this is evaluated?")
                vec![]
            }
        };
        // println!("cvec: {x:?}");
        x
    }

    fn to_var(&self) -> Option<egg::Symbol> {
        if let Aella::Var(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    fn mk_var(sym: egg::Symbol) -> Self {
        Aella::Var(sym)
    }

    fn to_constant(&self) -> Option<&Self::Constant> {
        // The only place that this function is used is for
        // `is_constant`. I'm just overriding that function
        // directly because I can't return a reference.
        unreachable!("I thought this wasn't used.")
    }

    fn mk_constant(c: Self::Constant) -> Option<Self> {
        let Value(val, _env) = c;
        val.map(|x| match x {
            PrimVal::Int(val) => Aella::Num(val),
            PrimVal::Var(name) => Aella::Var(name),
        })
    }

    fn is_constant(&self) -> bool {
        matches!(self, Aella::Num(..))
    }

    fn init_synth(synth: &mut ruler::Synthesizer<Self, ruler::Uninit>) {
        // initial constants that will be in the graph
        let constants = [0];

        let cvec_len = ruler::self_product(
            &constants
                .iter()
                .map(|x| Some(Value::prim(PrimVal::Int(*x))))
                .collect::<Vec<_>>(),
            synth.params.variables,
        )
        .len();

        println!("cvec_len: {cvec_len}");

        // make a new egraph
        let mut egraph = egg::EGraph::new(ruler::SynthAnalysis { cvec_len });

        // add constants to the egraph
        for v in constants {
            egraph.add(Aella::Num(v));
        }

        let vars: Vec<egg::Symbol> = (0..synth.params.variables)
            .map(|i| egg::Symbol::from(ruler::letter(i)))
            .collect();

        for var in &vars {
            let free_id = egraph.add(Aella::Var(*var));

            // ints
            egraph[free_id].data.cvec = (0..cvec_len / 2)
                .map(|_| Value::random_env(&mut synth.rng, 0, 10))
                .map(Some)
                .collect::<Vec<_>>();

            // vars
            egraph[free_id].data.cvec.extend(
                (0..cvec_len / 2)
                    .map(|_| Value::random_env(&mut synth.rng, 1, 10))
                    .map(Some)
                    .collect::<Vec<_>>(),
            );

            egraph[free_id].data.cvec.extend(
                (0..cvec_len / 2)
                    .map(|_| Value::random_env(&mut synth.rng, 2, 10))
                    .map(Some)
                    .collect::<Vec<_>>(),
            );

            // make the cvec use
            println!("cvec: {} {:?}", var, egraph[free_id].data.cvec);
        }

        synth.egraph = egraph;
    }

    fn make_layer<'a>(
        ids: Vec<Id>,
        synth: &'a ruler::Synthesizer<Self, ruler::Init>,
        _iter: usize,
    ) -> Box<dyn Iterator<Item = Self> + 'a> {
        let binops = (0..2)
            .map(|_| ids.clone())
            .multi_cartesian_product()
            .filter(move |ids| !ids.iter().all(|x| synth.egraph[*x].data.exact))
            .map(|ids| [ids[0], ids[1]])
            .map(move |x| {
                vec![
                    Aella::Plus(x),
                    Aella::Sub(x),
                    Aella::Times(x),
                    Aella::Assign(x),
                    Aella::Seq(x),
                    Aella::AsmMov(x),
                ]
            })
            .flatten();

        let terops = (0..3)
            .map(|_| ids.clone())
            .multi_cartesian_product()
            .filter(move |ids| !ids.iter().all(|x| synth.egraph[*x].data.exact))
            .map(|ids| [ids[0], ids[1], ids[2]])
            .map(move |x| {
                vec![Aella::AsmAdd(x), Aella::AsmSub(x), Aella::AsmSmull(x)]
            })
            .flatten();

        Box::new(binops.chain(terops))
    }

    fn is_valid(
        _synth: &mut ruler::Synthesizer<Self, ruler::Init>,
        lhs: &egg::Pattern<Self>,
        rhs: &egg::Pattern<Self>,
    ) -> bool {
        // construct an environment for the variables
        // let mut env = std::collections::HashMap::default();

        // for var in lhs.vars().into_iter().chain(rhs.vars().into_iter()) {
        //     let v: egg::Symbol = var.to_string().into();
        //     env.insert(
        //         var,
        //         (0..2)
        //             .map(|_| synth.rng.gen_range(-100, 100))
        //             .map(|n: i64| Value::Env(v, BTreeMap::from([(v, n)])))
        //             .map(Some)
        //             .collect::<Vec<_>>(),
        //     );
        // }

        // lhs
        // let lvec = match &lhs.ast.as_ref()[lhs.ast.as_ref().len() - 1] {
        //     egg::ENodeOrVar::ENode(node) => match node {
        //         Aella::Plus(_) => (),
        //         Aella::Sub(_) => (),
        //         Aella::Times(_) => (),
        //         Aella::Div(_) => (),
        //         Aella::Eq(_) => (),
        //         Aella::Lt(_) => (),
        //         Aella::Not(_) => (),
        //         Aella::And(_) => (),
        //         Aella::Seq(_) => (),
        //         Aella::Assign(_) => (),
        //         Aella::While(_) => (),
        //         Aella::AsmMov(_) => (),
        //         Aella::AsmAdd([rd, rn, rm]) => {
        //             // blah
        //             let rdn = &lhs.ast[*rd];
        //             let rnn = &lhs.ast[*rn];
        //             let rmn = &lhs.ast[*rm];

        //             let y = &synth.egraph[*rd].data.cvec;

        //             // eprintln!("{rdn} {y:?}");
        //             // let env = std::collections::HashMap::default();
        //             // Self::eval_pattern(lhs, &env, 10);
        //         }
        //         Aella::AsmSub(_) => (),
        //         Aella::AsmSmull(_) => (),
        //         Aella::AsmSdiv(_) => (),
        //         Aella::Num(_) => (),
        //         Aella::Var(_) => (),
        //     },
        //     egg::ENodeOrVar::Var(_) => unreachable!(),
        // };

        // let lvec = Self::eval_pattern(lhs, &env, 10);
        // let rvec = Self::eval_pattern(lhs, &env, 10);

        eprintln!("rule: {lhs} => {rhs}");
        // eprintln!("rhs: {lvec:?} ={}= {rvec:?}", lvec == rvec);
        // panic!("env: {:?}", env);

        true
    }
}

pub fn run(
    params: ruler::SynthParams,
    opts: crate::SynthOpts,
) -> ruler::Report<Aella> {
    let syn =
        ruler::Synthesizer::<Aella, _>::new_with_data(params.clone(), opts)
            .init();
    <Aella as ruler::SynthLanguage>::post_process(&params, syn.run())
}

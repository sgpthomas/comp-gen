use comp_gen::ruler;
use egg::{define_language, EGraph, Id, Language};
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
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::str::FromStr;
use z3::ast::{Ast, Bool, Datatype};

use crate::error::Error;
use crate::Res;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub enum Value {
    // starts with i
    Int(i32),
    // starts with [
    List(Vec<Value>),
    // starts with <
    Vec(Vec<Value>),
    // starts with b
    Bool(bool),
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.starts_with("i") {
            let v: i32 =
                s[1..].parse().map_err(|_| "Bad integer.".to_string())?;
            Ok(Value::Int(v))
        } else {
            Err(format!("{} didn't match anything.", s))
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(l) => write!(f, "{:?}", l),
            Value::Vec(v) => {
                write!(
                    f,
                    "(Vec {})",
                    v.into_iter()
                        .map(|x| format!("{}", x))
                        .collect_vec()
                        .join(" ")
                )
            }
        }
    }
}

fn split_into_halves(n: usize) -> (usize, usize) {
    if n % 2 == 0 {
        (n / 2, n / 2)
    } else {
        (n / 2, n / 2 + 1)
    }
}

impl Value {
    fn int1<F>(arg: &Self, f: F) -> Option<Value>
    where
        F: Fn(i32) -> Value,
    {
        if let Value::Int(val) = arg {
            Some(f(*val))
        } else {
            None
        }
    }

    fn int2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<Value>
    where
        F: Fn(i32, i32) -> Value,
    {
        if let (Value::Int(lv), Value::Int(rv)) = (lhs, rhs) {
            Some(f(*lv, *rv))
        } else {
            None
        }
    }

    fn bool2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<Value>
    where
        F: Fn(bool, bool) -> Value,
    {
        if let (Value::Bool(lv), Value::Bool(rv)) = (lhs, rhs) {
            Some(f(*lv, *rv))
        } else {
            None
        }
    }

    fn vec1<F>(val: &Self, f: F) -> Option<Value>
    where
        F: Fn(&[Value]) -> Option<Value>,
    {
        if let Value::Vec(v) = val {
            f(v)
        } else {
            None
        }
    }

    fn vec2<F>(lhs: &Self, rhs: &Self, f: F) -> Option<Value>
    where
        F: Fn(&[Value], &[Value]) -> Option<Value>,
    {
        if let (Value::Vec(v1), Value::Vec(v2)) = (lhs, rhs) {
            if v1.len() == v2.len() {
                f(v1, v2)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn vec3<F>(v1: &Self, v2: &Self, v3: &Self, f: F) -> Option<Value>
    where
        F: Fn(&[Value], &[Value], &[Value]) -> Option<Value>,
    {
        if let (Value::Vec(v1), Value::Vec(v2), Value::Vec(v3)) = (v1, v2, v3) {
            if v1.len() == v2.len() && v2.len() == v3.len() {
                f(v1, v2, v3)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn vec2_op<F>(lhs: &Self, rhs: &Self, f: F) -> Option<Value>
    where
        F: Fn(&Value, &Value) -> Option<Value>,
    {
        Self::vec2(lhs, rhs, |lhs, rhs| {
            lhs.iter()
                .zip(rhs)
                .map(|(l, r)| f(l, r))
                .collect::<Option<Vec<Value>>>()
                .map(|v| Value::Vec(v))
        })
    }

    #[allow(unused)]
    fn int_range(min: i32, max: i32, num_samples: usize) -> Vec<Value> {
        (min..=max)
            .step_by(((max - min) as usize) / num_samples)
            .map(|x| Value::Int(x))
            .collect::<Vec<_>>()
    }

    fn sample_int(
        rng: &mut Pcg64,
        min: i32,
        max: i32,
        num_samples: usize,
    ) -> Vec<Value> {
        (0..num_samples)
            .map(|_| Value::Int(rng.gen_range(min, max)))
            .collect::<Vec<_>>()
    }

    fn sample_vec(
        rng: &mut Pcg64,
        min: i32,
        max: i32,
        list_size: usize,
        num_samples: usize,
    ) -> Vec<Value> {
        (0..num_samples)
            .map(|_| Value::Vec(Value::sample_int(rng, min, max, list_size)))
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

define_language! {
    pub enum VecLang {
        // Id is a key to identify EClasses within an EGraph, represents
        // children nodes
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Minus([Id; 2]),
        "/" = Div([Id; 2]),

        "or" = Or([Id; 2]),
        "&&" = And([Id; 2]),
        "ite" = Ite([Id; 3]),
        "<" = Lt([Id; 2]),

        "sgn" = Sgn([Id; 1]),
        "sqrt" = Sqrt([Id; 1]),
        "neg" = Neg([Id; 1]),

        // Lists have a variable number of elements
        "List" = List(Box<[Id]>),

        // Vectors have width elements
        "Vec" = Vec(Box<[Id]>),

        // Vector with all literals
        // "LitVec" = LitVec(Box<[Id]>),

        "Get" = Get([Id; 2]),

        // Used for partitioning and recombining lists
        "Concat" = Concat([Id; 2]),

        // Vector operations that take 2 vectors of inputs
        "VecAdd" = VecAdd([Id; 2]),
        "VecMinus" = VecMinus([Id; 2]),
        "VecMul" = VecMul([Id; 2]),
        "VecDiv" = VecDiv([Id; 2]),
        "VecMulSgn" = VecMulSgn([Id; 2]),

        // Vector operations that take 1 vector of inputs
        "VecNeg" = VecNeg([Id; 1]),
        "VecSqrt" = VecSqrt([Id; 1]),
        "VecSgn" = VecSgn([Id; 1]),

        // MAC takes 3 lists: acc, v1, v2
        "VecMAC" = VecMAC([Id; 3]),

    // Comment
        Const(Value),

        // language items are parsed in order, and we want symbol to
        // be a fallback, so we put it last.
        // `Symbol` is an egg-provided interned string type
        Symbol(egg::Symbol),
    }
}

#[derive(Debug, Clone)]
enum Lang {
    Add(Box<Lang>, Box<Lang>),
    Mul(Box<Lang>, Box<Lang>),
    Minus(Box<Lang>, Box<Lang>),
    Div(Box<Lang>, Box<Lang>),

    Or(Box<Lang>, Box<Lang>),
    And(Box<Lang>, Box<Lang>),
    #[allow(unused)]
    Ite(Box<Lang>, Box<Lang>, Box<Lang>),
    Lt(Box<Lang>, Box<Lang>),

    Neg(Box<Lang>),

    Vec(Vec<Lang>),

    VecAdd(Box<Lang>, Box<Lang>),
    VecMul(Box<Lang>, Box<Lang>),
    VecMinus(Box<Lang>, Box<Lang>),
    VecDiv(Box<Lang>, Box<Lang>),

    VecNeg(Box<Lang>),
    #[allow(unused)]
    VecSqrt(Box<Lang>),
    #[allow(unused)]
    VecSgn(Box<Lang>),

    VecMAC(Box<Lang>, Box<Lang>, Box<Lang>),

    Const(Value),
    Symbol(String),
}

impl Lang {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<VecLang>) -> Id {
        match &self {
            Lang::Add(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Add([left_id, right_id]))
            }
            Lang::Mul(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Mul([left_id, right_id]))
            }
            Lang::Minus(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Minus([left_id, right_id]))
            }
            Lang::Div(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Div([left_id, right_id]))
            }
            Lang::Or(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Or([left_id, right_id]))
            }
            Lang::And(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::And([left_id, right_id]))
            }
            Lang::Lt(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Lt([left_id, right_id]))
            }
            Lang::VecAdd(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecAdd([left_id, right_id]))
            }
            Lang::VecMul(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecMul([left_id, right_id]))
            }
            Lang::VecMinus(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecMinus([left_id, right_id]))
            }
            Lang::VecDiv(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecDiv([left_id, right_id]))
            }
            Lang::Ite(_, _, _) => todo!(),
            Lang::Neg(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::Neg([id]))
            }
            Lang::VecNeg(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecNeg([id]))
            }
            Lang::VecSqrt(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecSqrt([id]))
            }
            Lang::VecSgn(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecSgn([id]))
            }
            Lang::VecMAC(a, b, c) => {
                let a_id = a.to_recexpr(expr);
                let b_id = b.to_recexpr(expr);
                let c_id = c.to_recexpr(expr);
                expr.add(VecLang::VecMAC([a_id, b_id, c_id]))
            }
            Lang::Vec(items) => {
                let ids =
                    items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
                expr.add(VecLang::Vec(ids.into_boxed_slice()))
            }
            Lang::Const(v) => expr.add(VecLang::Const(v.clone())),
            Lang::Symbol(s) => expr.add(VecLang::Symbol(s.into())),
        }
    }
}

fn subtree(
    expr: &egg::RecExpr<VecLang>,
    new_root: Id,
) -> egg::RecExpr<VecLang> {
    expr[new_root].build_recexpr(|id| expr[id].clone())
}

impl From<egg::RecExpr<VecLang>> for Lang {
    fn from(expr: egg::RecExpr<VecLang>) -> Self {
        let root: Id = Id::from(expr.as_ref().len() - 1);
        match &expr[root] {
            VecLang::Add([left, right]) => Lang::Add(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Mul([left, right]) => Lang::Mul(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Minus([left, right]) => Lang::Minus(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Div([left, right]) => Lang::Div(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Or([left, right]) => Lang::Or(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::And([left, right]) => Lang::And(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Ite(_) => todo!(),
            VecLang::Lt([left, right]) => Lang::Lt(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Sgn(_) => todo!(),
            VecLang::Sqrt(_) => todo!(),
            VecLang::Neg([inner]) => {
                Lang::Neg(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::List(_) => todo!(),
            VecLang::Vec(items) => Lang::Vec(
                items
                    .iter()
                    .map(|id| subtree(&expr, *id).into())
                    .collect_vec(),
            ),
            VecLang::Get(_) => todo!(),
            VecLang::Concat(_) => todo!(),
            VecLang::VecAdd([left, right]) => Lang::VecAdd(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMinus([left, right]) => Lang::VecMinus(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMul([left, right]) => Lang::VecMul(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecDiv([left, right]) => Lang::VecDiv(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMulSgn(_) => todo!(),
            VecLang::VecNeg([inner]) => {
                Lang::VecNeg(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::VecSqrt(_) => todo!(),
            VecLang::VecSgn(_) => todo!(),
            VecLang::VecMAC([a, b, c]) => Lang::VecMAC(
                Box::new(subtree(&expr, *a).into()),
                Box::new(subtree(&expr, *b).into()),
                Box::new(subtree(&expr, *c).into()),
            ),
            VecLang::Const(v) => Lang::Const(v.clone()),
            VecLang::Symbol(sym) => Lang::Symbol(sym.to_string()),
        }
    }
}

impl Into<egg::RecExpr<VecLang>> for Lang {
    fn into(self) -> egg::RecExpr<VecLang> {
        let mut expr = egg::RecExpr::default();
        self.to_recexpr(&mut expr);
        expr
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
    pub use_vector: bool,
    pub vector_mac: bool,
    pub variable_duplication: bool,
    pub vector_size: usize,
}

impl Default for DiosConfig {
    fn default() -> Self {
        Self {
            constants: vec![],
            seed_rules: vec![],
            unops: vec![],
            binops: vec![],
            use_vector: false,
            vector_mac: false,
            variable_duplication: false,
            vector_size: 1,
        }
    }
}

impl SynthLanguage for VecLang {
    type Constant = Value;
    type Config = DiosConfig;

    fn to_var(&self) -> Option<egg::Symbol> {
        if let VecLang::Symbol(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    fn mk_var(sym: egg::Symbol) -> Self {
        VecLang::Symbol(sym)
    }

    fn to_constant(&self) -> Option<&Self::Constant> {
        if let VecLang::Const(n) = self {
            Some(n)
        } else {
            None
        }
    }

    fn mk_constant(c: <Self as SynthLanguage>::Constant) -> Option<Self> {
        Some(VecLang::Const(c))
    }

    fn eval<'a, F>(&'a self, cvec_len: usize, mut get: F) -> CVec<Self>
    where
        F: FnMut(&'a Id) -> &'a CVec<Self>,
    {
        match self {
            VecLang::Const(i) => vec![Some(i.clone()); cvec_len],
            VecLang::Add([l, r]) => {
                map!(get, l, r => Value::int2(l, r, |l, r| Value::Int(l + r)))
            }
            VecLang::Mul([l, r]) => {
                map!(get, l, r => Value::int2(l, r, |l, r| Value::Int(l * r)))
            }
            VecLang::Minus([l, r]) => {
                map!(get, l, r => Value::int2(l, r, |l, r| Value::Int(l - r)))
            }
            VecLang::Div([l, r]) => get(l)
                .iter()
                .zip(get(r).iter())
                .map(|tup| match tup {
                    (Some(Value::Int(a)), Some(Value::Int(b))) => {
                        if *b != 0 {
                            if *a == 0 {
                                Some(Value::Int(0))
                            } else if a >= b {
                                Some(Value::Int(a / b))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            VecLang::Or([l, r]) => {
                map!(get, l, r => Value::bool2(l, r, |l, r| Value::Bool(l || r)))
            }
            VecLang::And([l, r]) => {
                map!(get, l, r => Value::bool2(l, r, |l, r| Value::Bool(l && r)))
            }
            VecLang::Ite([_b, _t, _f]) => todo!(),
            VecLang::Lt([l, r]) => {
                map!(get, l, r => Value::int2(l, r, |l, r| Value::Bool(l < r)))
            }
            VecLang::Sgn([x]) => {
                map!(get, x => Value::int1(x, |x| Value::Int(sgn(x))))
            }
            VecLang::Sqrt([x]) => get(x)
                .iter()
                .map(|a| match a {
                    Some(Value::Int(a)) => {
                        if *a >= 0 {
                            Some(Value::Int(a.sqrt()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            VecLang::Neg([x]) => {
                map!(get, x => Value::int1(x, |x| Value::Int(-x)))
            }
            VecLang::List(l) => l
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
                        Some(Value::List(x))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            VecLang::Vec(l) => l
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
                        Some(Value::Vec(x))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            // VecLang::LitVec(x) => todo!(),
            VecLang::Get([l, i]) => map!(get, l, i => {
                if let (Value::Vec(v), Value::Int(idx)) = (l, i) {
            // get index and clone the inner Value if there is one
                    v.get(*idx as usize).map(|inner| inner.clone())
                } else {
            None
            }
                }),
            VecLang::Concat([l, r]) => {
                map!(get, l, r => Value::vec2(l, r, |l, r| {
                Some(Value::List(l.iter().chain(r).cloned().collect::<Vec<_>>()))
                    }))
            }
            VecLang::VecAdd([l, r]) => {
                map!(get, l, r => Value::vec2_op(l, r, |l, r| Value::int2(l, r, |l, r| Value::Int(l + r))))
            }
            VecLang::VecMinus([l, r]) => {
                map!(get, l, r => Value::vec2_op(l, r, |l, r| Value::int2(l, r, |l, r| Value::Int(l - r))))
            }
            VecLang::VecMul([l, r]) => {
                map!(get, l, r => Value::vec2_op(l, r, |l, r| Value::int2(l, r, |l, r| Value::Int(l * r))))
            }
            VecLang::VecDiv([l, r]) => {
                map!(get, l, r => Value::vec2_op(l, r, |l, r| {
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => {
                        if *b != 0 {
                            Some(Value::Int(a / b))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
                }))
            }
            VecLang::VecMulSgn(_) => todo!(),
            VecLang::VecNeg([l]) => {
                map!(get, l => Value::vec1(l, |l| {
                        if l.iter().all(|x| matches!(x, Value::Int(_))) {
                        Some(Value::Vec(l.iter().map(|tup| match tup {
                                Value::Int(a) => Value::Int(-a),
                                    x => panic!("NEG: Ill-formed: {}", x)
                                }).collect::<Vec<_>>()))
                        } else {
                    None
                }
                            }))
            }
            VecLang::VecSqrt([l]) => {
                map!(get, l => Value::vec1(l, |l| {
                    if l.iter().all(|x| {if let Value::Int(i) = x { *i >= 0 } else { false }}) {
                        Some(Value::Vec(l.iter().map(|tup| match tup {
                                Value::Int(a) => Value::Int(a.sqrt()),
                                    x => panic!("SQRT: Ill-formed: {}", x)
                                }).collect::<Vec<_>>()))
                        } else {
                    None
                }
                            }))
            }
            VecLang::VecSgn([l]) => {
                map!(get, l => Value::vec1(l, |l| {
                        if l.iter().all(|x| matches!(x, Value::Int(_))) {
                        Some(Value::Vec(l.iter().map(|tup| match tup {
                                Value::Int(a) => Value::Int(sgn(*a)),
                                    x => panic!("SGN: Ill-formed: {}", x)
                                }).collect::<Vec<_>>()))
                        } else {
                    None
                }
                            }))
            }
            VecLang::VecMAC([acc, v1, v2]) => {
                map!(get, v1, v2, acc => Value::vec3(v1, v2, acc, |v1, v2, acc| {
                            v1.iter().zip(v2.iter()).zip(acc.iter()).map(|tup| match tup {
                    ((Value::Int(v1), Value::Int(v2)), Value::Int(acc)) => Some(Value::Int((v1 * v2) + acc)),
                _ => None
                            }).collect::<Option<Vec<Value>>>()
                .map(|x| Value::Vec(x))
                        }))
            }
            VecLang::Symbol(_) => vec![],
        }
    }

    fn init_synth(synth: &mut Synthesizer<Self, ruler::Uninit>) {
        let consts = synth
            .lang_config
            .constants
            .iter()
            .map(|c| match c.kind.as_str() {
                "int" => Value::Int(c.value),
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
            let rule: Equality<VecLang> = Equality::new(
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
            egraph.add(VecLang::Const(v));
        }

        // add variables
        for i in 0..synth.params.variables {
            let var = egg::Symbol::from(letter(i));
            let id = egraph.add(VecLang::Symbol(var));

            // make the cvec use real data
            let mut cvec = vec![];

            let (n_ints, n_vecs) = split_into_halves(cvec_size);

            cvec.extend(
                Value::sample_int(&mut synth.rng, -100, 100, n_ints)
                    .into_iter()
                    .map(Some),
            );

            cvec.extend(
                Value::sample_vec(
                    &mut synth.rng,
                    -100,
                    100,
                    4, // synth.params.vector_size
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
        let binops = if iter < 2 {
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
                            "neg" => VecLang::Neg([x]),
                            "sgn" => VecLang::Sgn([x]),
                            "sqrt" => VecLang::Sqrt([x]),
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
                            "+" => VecLang::Add(x),
                            "*" => VecLang::Mul(x),
                            "-" => VecLang::Minus(x),
                            "/" => VecLang::Div(x),
                            "or" => VecLang::Or(x),
                            "&&" => VecLang::And(x),
                            "<" => VecLang::Lt(x),
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
                            "neg" => VecLang::VecNeg([x]),
                            "sgn" => VecLang::VecSgn([x]),
                            "sqrt" => VecLang::VecSqrt([x]),
                            _ => panic!("Unknown vec unop"),
                        })
                        .collect_vec();
                    v.extend(vec![VecLang::Vec(vec![x].into_boxed_slice())]);
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
                            "+" => VecLang::VecAdd(x),
                            "-" => VecLang::VecMinus(x),
                            "*" => VecLang::VecMul(x),
                            "/" => VecLang::VecDiv(x),
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
                .map(move |x| VecLang::VecMAC(x))
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
        let mut cfg = z3::Config::new();
        cfg.set_timeout_msec(1000);
        let ctx = z3::Context::new(&cfg);
        let solver = z3::Solver::new(&ctx);

        let left = egg_to_z3(&ctx, Self::instantiate(lhs).as_ref());
        let right = egg_to_z3(&ctx, Self::instantiate(rhs).as_ref());

        let ret = if let (Some((lexpr, lasses)), Some((rexpr, rasses))) =
            (left, right)
        {
            solver.reset();
            let all: Vec<_> =
                lasses.into_iter().chain(rasses.into_iter()).collect();
            let qe_eq = &lexpr._eq(&rexpr).not();
            // log::info!("z3 eq: {} = {}", lhs, rhs);
            // log::info!("z3 asses: {:?}", all);
            // for ass in &all {
            //     solver.reset();
            //     solver.assert(ass);
            //     match solver.check() {
            //         z3::SatResult::Unsat => {
            //             log::info!("z3 ass was unsat {}", ass);
            //             return false;
            //         }
            //         z3::SatResult::Unknown => {
            //             log::info!("z3 ass was unknown {}", ass);
            //             return false;
            //         }
            //         z3::SatResult::Sat => {
            //             continue;
            //         }
            //     }
            // }

            solver.assert(qe_eq);
            log::debug!("z3 check {} != {}", lhs, rhs);

            match solver.check_assumptions(&all) {
                z3::SatResult::Sat => {
                    log::debug!("counter-example: {:?}", solver.get_model());
                    false
                }
                z3::SatResult::Unsat => {
                    log::debug!(
                        "z3 validation: success for {} => {}",
                        lhs,
                        rhs
                    );
                    log::debug!("core: {:?}", solver.get_unsat_core());
                    true
                }
                z3::SatResult::Unknown => {
                    synth.smt_unknown += 1;
                    log::debug!(
                        "z3 validation: unknown for {} => {}",
                        lhs,
                        rhs
                    );
                    false
                    // vecs_eq(&lvec, &rvec)
                }
            }
        } else {
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

            let possibilities = vec![-5, -2, -1, 0, 1, 2, 5];
            // let possibilities = vec![0, 1, 2];
            for perms in possibilities.iter().permutations(env.keys().len()) {
                for (i, cvec) in perms.iter().zip(env.values_mut()) {
                    cvec.push(Some(Value::Int(**i)));
                }
            }

            // add some random vectors
            let mut length = 0;
            for cvec in env.values_mut() {
                cvec.extend(
                    Value::sample_vec(
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

            // debug(
            //     "(/ ?b ?a)",
            //     "(VecMul ?b ?b)",
            //     length,
            //     &[("?a", Value::Int(10)), ("?b", Value::Int(20))],
            // );
            // panic!();

            let lvec = Self::eval_pattern(lhs, &env, length);
            let rvec = Self::eval_pattern(rhs, &env, length);

            if lvec != rvec {
                log::debug!("  env: {:?}", env);
                log::debug!("  lhs: {}, rhs: {}", lhs, rhs);
                log::debug!("  lvec: {:?}, rvec: {:?}", lvec, rvec);
                log::debug!("  eq: {}", vecs_eq(&lvec, &rvec));
            }

            vecs_eq(&lvec, &rvec)
        };
        // only compare values where both sides are defined
        ret
    }

    // fn post_process(
    //     params: &SynthParams,
    //     mut report: ruler::Report<Self>,
    // ) -> ruler::Report<Self> {
    //     let mut new_eqs: Vec<Equality<_>> = vec![];
    //     for eq in &report.eqs {
    //         let lhs_pre: Lang = eq.lhs.unpattern().into();
    //         let lhs: egg::RecExpr<VecLang> =
    //             lhs_pre.desugar(params.vector_size).into();
    //         let rhs_pre: Lang = eq.rhs.unpattern().into();
    //         let rhs: egg::RecExpr<VecLang> =
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
    node: &VecLang,
    egraph: &EGraph<VecLang, SynthAnalysis>,
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
    node: &VecLang,
    egraph: &EGraph<VecLang, SynthAnalysis>,
) -> bool {
    let vars: Vec<egg::Symbol> = get_vars(node, egraph);
    vars.iter().all_unique()
}

fn vecs_eq(lvec: &CVec<VecLang>, rvec: &CVec<VecLang>) -> bool {
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
fn debug(left: &str, right: &str, n: usize, env_pairs: &[(&str, Value)]) {
    let mut env: HashMap<
        egg::Var,
        Vec<Option<Value>>,
        BuildHasherDefault<FxHasher>,
    > = HashMap::default();

    env_pairs.iter().for_each(|(var, value)| {
        env.insert(egg::Var::from_str(var).unwrap(), vec![Some(value.clone())]);
    });

    let pleft: egg::Pattern<VecLang> = left.parse().unwrap();
    let pright: egg::Pattern<VecLang> = right.parse().unwrap();
    log::info!("left");
    let lres = VecLang::eval_pattern(&pleft, &env, n);
    log::info!("right");
    let rres = VecLang::eval_pattern(&pright, &env, n);
    log::info!(
        "TEST:\n  {lres:?}\n    ?= ({})\n  {rres:?}",
        vecs_eq(&lres, &rres),
    );
    log::info!("{} => {}", left, right);
}

pub fn run(
    params: ruler::SynthParams,
    opts: crate::SynthOpts,
) -> Res<ruler::Report<VecLang>> {
    // parse the configuration
    let dios_config: DiosConfig = opts
        .config
        .map(|config| {
            serde_json::from_reader(File::open(config)?)
                .map_err(|err| Error::from(err))
        })
        .unwrap_or_else(|| Ok(DiosConfig::default()))?;
    // create the synthesizer
    let syn = ruler::Synthesizer::<VecLang, _>::new_with_data(
        params.clone(),
        dios_config,
    )
    .init();
    // run the synthesizer
    let report = syn.run();

    Ok(VecLang::post_process(&params, report))
}

fn egg_to_z3<'a>(
    _ctx: &'a z3::Context,
    _expr: &[VecLang],
) -> Option<(Datatype<'a>, Vec<Bool<'a>>)> {
    // let mut buf: Vec<Datatype> = vec![];
    // let mut assumes: Vec<Bool> = vec![];

    // // data type representing either ints or bools
    // let int_bool = DatatypeBuilder::new(&ctx, "Int+Bool")
    //     .variant(
    //         "Int",
    //         vec![("Int.v", DatatypeAccessor::Sort(Sort::int(&ctx)))],
    //     )
    //     .variant(
    //         "Bool",
    //         vec![("Bool.v", DatatypeAccessor::Sort(Sort::bool(&ctx)))],
    //     )
    //     .finish();

    // let int_cons = &int_bool.variants[0].constructor;
    // let int_get = &int_bool.variants[0].accessors[0];
    // let bool_cons = &int_bool.variants[1].constructor;
    // let _bool_get = &int_bool.variants[1].accessors[0];

    // let mut vars: Vec<_> = vec![];

    // for node in expr.as_ref().iter() {
    //     match node {
    //         VecLang::Const(Value::Int(i)) => buf.push(
    //             int_cons
    //                 .apply(&[&Int::from_i64(&ctx, *i as i64)])
    //                 .as_datatype()
    //                 .unwrap(),
    //         ),
    //         VecLang::Symbol(v) => {
    //             let var = Datatype::new_const(&ctx, v.to_string(), &int_bool.sort);
    //             vars.push(buf.len());
    //             buf.push(var)
    //         }
    //         VecLang::Add([x, y]) => {
    //             let x_int = int_get.apply(&[&buf[usize::from(*x)]]).as_int().unwrap();
    //             // println!("buf: {:?}[{}]", buf, y);
    //             let y_int = int_get.apply(&[&buf[usize::from(*y)]]).as_int().unwrap();
    //             let res = int_cons.apply(&[&(x_int + y_int)]).as_datatype().unwrap();
    //             buf.push(res)
    //         }
    //         VecLang::Mul([x, y]) => {
    //             let x_int = int_get.apply(&[&buf[usize::from(*x)]]).as_int().unwrap();
    //             let y_int = int_get.apply(&[&buf[usize::from(*y)]]).as_int().unwrap();
    //             let res = int_cons.apply(&[&(x_int * y_int)]).as_datatype().unwrap();
    //             buf.push(res)
    //         }
    //         VecLang::Div([x, y]) => {
    //             let denom = int_get.apply(&[&buf[usize::from(*y)]]).as_int().unwrap();
    //             // let eqz = Int::le(&denom, &Int::from_i64(&ctx, 0));
    //             let lez = Int::le(&denom, &Int::from_i64(&ctx, 0));
    //             let gez = Int::ge(&denom, &Int::from_i64(&ctx, 0));
    //             let assume = Bool::not(&Bool::and(&ctx, &[&lez, &gez]));
    //             assumes.push(assume);
    //             let x_int = int_get.apply(&[&buf[usize::from(*x)]]).as_int().unwrap();
    //             let term = Int::div(&x_int, &denom);

    //             buf.push(int_cons.apply(&[&term]).as_datatype().unwrap())
    //         }
    //         VecLang::Lt([a, b]) => {
    //             let a_int = int_get.apply(&[&buf[usize::from(*a)]]).as_int().unwrap();
    //             let b_int = int_get.apply(&[&buf[usize::from(*b)]]).as_int().unwrap();
    //             let term = Int::lt(&a_int, &b_int);

    //             // let pattern = z3::Pattern::new(&ctx, &[&term]);
    //             // let vs: Vec<&dyn Ast> = vars.iter().map(|i| &buf[usize::from(*i)] as _).collect();
    //             // let forall: Bool = forall_const(&ctx, vs.as_slice(), &[&pattern], &term);

    //             let res = bool_cons.apply(&[&term]).as_datatype().unwrap();

    //             // assumes.push(forall);
    //             buf.push(res)
    //         }
    //         _ => (),
    //     }
    // }

    // buf.pop().map(|head| (head, assumes))
    None
}

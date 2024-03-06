use comp_gen::ruler::egg::{self, define_language, Id, Language};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(
    Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Serialize, Deserialize,
)]
pub enum Value {
    // starts with i
    Int(i64),
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
        let int = str::parse::<i64>(s).map_err(|e| e.to_string())?;
        Ok(Value::Int(int))
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
                    v.iter().map(|x| format!("{}", x)).collect_vec().join(" ")
                )
            }
        }
    }
}

define_language! {
    #[derive(Serialize, Deserialize)]
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
        "sqrtsgn" = SqrtSgn([Id; 2]),

        "sgn" = Sgn([Id; 1]),
        "sqrt" = Sqrt([Id; 1]),
        "neg" = Neg([Id; 1]),

        // Lists have a variable number of elements
        "List" = List(Box<[Id]>),

        // Vectors have width elements
        "Vec" = Vec(Box<[Id]>),

        // Vector with all literals
        "LitVec" = LitVec(Box<[Id]>),

        "Get" = Get([Id; 2]),

        // Used for partitioning and recombining lists
        "Concat" = Concat([Id; 2]),

        // Vector operations that take 2 vectors of inputs
        "VecAdd" = VecAdd([Id; 2]),
        "VecMinus" = VecMinus([Id; 2]),
        "VecMul" = VecMul([Id; 2]),
        "VecDiv" = VecDiv([Id; 2]),
        "VecMulSgn" = VecMulSgn([Id; 2]),
        "VecSqrtSgn" = VecSqrtSgn([Id; 2]),

        // Vector operations that take 1 vector of inputs
        "VecNeg" = VecNeg([Id; 1]),
        "VecSqrt" = VecSqrt([Id; 1]),
        "VecSgn" = VecSgn([Id; 1]),
        // "VecRAdd" = VecRAdd([Id; 1]),

        // MAC takes 3 lists: acc, v1, v2
        "VecMAC" = VecMAC([Id; 3]),
        "VecMULS" = VecMULS([Id; 3]),

        // let binding
        "let" = Let([Id; 2]),

        // constant value
        Const(Value),

        // language items are parsed in order, and we want symbol to
        // be a fallback, so we put it last.
        // `Symbol` is an egg-provided interned string type
        Symbol(egg::Symbol),
    }
}

#[derive(Debug, Clone)]
pub enum VecAst {
    Add(Box<VecAst>, Box<VecAst>),
    Mul(Box<VecAst>, Box<VecAst>),
    Minus(Box<VecAst>, Box<VecAst>),
    Div(Box<VecAst>, Box<VecAst>),

    Or(Box<VecAst>, Box<VecAst>),
    And(Box<VecAst>, Box<VecAst>),
    #[allow(unused)]
    Ite(Box<VecAst>, Box<VecAst>, Box<VecAst>),
    Lt(Box<VecAst>, Box<VecAst>),
    SqrtSgn(Box<VecAst>, Box<VecAst>),
    Get(Box<VecAst>, Box<VecAst>),
    Concat(Box<VecAst>, Box<VecAst>),

    Sqrt(Box<VecAst>),
    Sgn(Box<VecAst>),
    Neg(Box<VecAst>),

    List(Vec<VecAst>),
    Vec(Vec<VecAst>),
    LitVec(Vec<VecAst>),

    VecAdd(Box<VecAst>, Box<VecAst>),
    VecMul(Box<VecAst>, Box<VecAst>),
    VecMinus(Box<VecAst>, Box<VecAst>),
    VecDiv(Box<VecAst>, Box<VecAst>),
    VecMulSgn(Box<VecAst>, Box<VecAst>),
    VecSqrtSgn(Box<VecAst>, Box<VecAst>),

    VecNeg(Box<VecAst>),
    VecSqrt(Box<VecAst>),
    VecSgn(Box<VecAst>),

    VecMAC(Box<VecAst>, Box<VecAst>, Box<VecAst>),
    VecMULS(Box<VecAst>, Box<VecAst>, Box<VecAst>),

    Let(String, Box<VecAst>),

    Const(Value),
    Symbol(String),
}

/// A simple visitor for VecAst
pub trait Visitor {
    fn do_pass(&mut self, ast: VecAst) -> VecAst {
        match ast {
            VecAst::Add(l, r) => self.visit_add(*l, *r),
            VecAst::Mul(l, r) => self.visit_mul(*l, *r),
            VecAst::Minus(l, r) => self.visit_minus(*l, *r),
            VecAst::Div(l, r) => self.visit_div(*l, *r),
            VecAst::Or(l, r) => self.visit_or(*l, *r),
            VecAst::And(l, r) => self.visit_and(*l, *r),
            VecAst::Ite(a, b, c) => self.visit_ite(*a, *b, *c),
            VecAst::Lt(l, r) => self.visit_lt(*l, *r),
            VecAst::SqrtSgn(l, r) => self.visit_sqrt_sgn(*l, *r),
            VecAst::Get(l, r) => self.visit_get(*l, *r),
            VecAst::Concat(l, r) => self.visit_concat(*l, *r),
            VecAst::Sqrt(x) => self.visit_sqrt(*x),
            VecAst::Sgn(x) => self.visit_sgn(*x),
            VecAst::Neg(x) => self.visit_neg(*x),
            VecAst::List(x) => self.visit_list(x),
            VecAst::Vec(x) => self.visit_vec(x),
            VecAst::LitVec(x) => self.visit_lit_vec(x),
            VecAst::VecAdd(l, r) => self.visit_vec_add(*l, *r),
            VecAst::VecMul(l, r) => self.visit_vec_mul(*l, *r),
            VecAst::VecMinus(l, r) => self.visit_vec_minus(*l, *r),
            VecAst::VecDiv(l, r) => self.visit_vec_div(*l, *r),
            VecAst::VecMulSgn(l, r) => self.visit_vec_mulsgn(*l, *r),
            VecAst::VecSqrtSgn(l, r) => self.visit_vec_sqrtsgn(*l, *r),
            VecAst::VecNeg(x) => self.visit_vec_neg(*x),
            VecAst::VecSqrt(x) => self.visit_vec_sqrt(*x),
            VecAst::VecSgn(x) => self.visit_vec_sgn(*x),
            VecAst::VecMAC(a, b, c) => self.visit_vec_mac(*a, *b, *c),
            VecAst::VecMULS(a, b, c) => self.visit_vec_muls(*a, *b, *c),
            VecAst::Let(l, r) => self.visit_let(l, *r),
            VecAst::Const(x) => self.visit_const(x),
            VecAst::Symbol(x) => self.visit_symbol(x),
        }
    }

    fn visit_add(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Add(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_mul(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Mul(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_minus(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Minus(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_div(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Div(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_or(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Or(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_and(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::And(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_ite(&mut self, cond: VecAst, tru: VecAst, fls: VecAst) -> VecAst {
        VecAst::Ite(
            Box::new(self.do_pass(cond)),
            Box::new(self.do_pass(tru)),
            Box::new(self.do_pass(fls)),
        )
    }

    fn visit_lt(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Lt(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_sqrt_sgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::SqrtSgn(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_get(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Get(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
    }

    fn visit_concat(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::Concat(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_sqrt(&mut self, inner: VecAst) -> VecAst {
        VecAst::Sqrt(Box::new(self.do_pass(inner)))
    }

    fn visit_sgn(&mut self, inner: VecAst) -> VecAst {
        VecAst::Sgn(Box::new(self.do_pass(inner)))
    }

    fn visit_neg(&mut self, inner: VecAst) -> VecAst {
        VecAst::Neg(Box::new(self.do_pass(inner)))
    }

    fn visit_list(&mut self, elems: Vec<VecAst>) -> VecAst {
        VecAst::List(elems.into_iter().map(|x| self.do_pass(x)).collect())
    }

    fn visit_vec(&mut self, elems: Vec<VecAst>) -> VecAst {
        VecAst::Vec(elems.into_iter().map(|x| self.do_pass(x)).collect())
    }

    fn visit_lit_vec(&mut self, elems: Vec<VecAst>) -> VecAst {
        VecAst::LitVec(elems.into_iter().map(|x| self.do_pass(x)).collect())
    }

    fn visit_vec_add(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecAdd(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_mul(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecMul(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_minus(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecMinus(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_div(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecDiv(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_mulsgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecMulSgn(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_sqrtsgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
        VecAst::VecSqrtSgn(
            Box::new(self.do_pass(left)),
            Box::new(self.do_pass(right)),
        )
    }

    fn visit_vec_neg(&mut self, inner: VecAst) -> VecAst {
        VecAst::VecNeg(Box::new(self.do_pass(inner)))
    }

    fn visit_vec_sqrt(&mut self, inner: VecAst) -> VecAst {
        VecAst::VecSqrt(Box::new(self.do_pass(inner)))
    }

    fn visit_vec_sgn(&mut self, inner: VecAst) -> VecAst {
        VecAst::VecSgn(Box::new(self.do_pass(inner)))
    }

    fn visit_vec_mac(&mut self, a: VecAst, b: VecAst, c: VecAst) -> VecAst {
        VecAst::VecMAC(
            Box::new(self.do_pass(a)),
            Box::new(self.do_pass(b)),
            Box::new(self.do_pass(c)),
        )
    }

    fn visit_vec_muls(&mut self, a: VecAst, b: VecAst, c: VecAst) -> VecAst {
        VecAst::VecMULS(
            Box::new(self.do_pass(a)),
            Box::new(self.do_pass(b)),
            Box::new(self.do_pass(c)),
        )
    }

    fn visit_let(&mut self, name: String, val: VecAst) -> VecAst {
        VecAst::Let(name, Box::new(self.do_pass(val)))
    }

    fn visit_const(&mut self, val: Value) -> VecAst {
        VecAst::Const(val)
    }

    fn visit_symbol(&mut self, symbol: String) -> VecAst {
        VecAst::Symbol(symbol)
    }
}

impl VecAst {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<VecLang>) -> Id {
        match self {
            VecAst::Add(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Add([left_id, right_id]))
            }
            VecAst::Mul(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Mul([left_id, right_id]))
            }
            VecAst::Minus(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Minus([left_id, right_id]))
            }
            VecAst::Div(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Div([left_id, right_id]))
            }
            VecAst::Or(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Or([left_id, right_id]))
            }
            VecAst::And(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::And([left_id, right_id]))
            }
            VecAst::Lt(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Lt([left_id, right_id]))
            }
            VecAst::SqrtSgn(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::SqrtSgn([left_id, right_id]))
            }
            VecAst::VecAdd(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecAdd([left_id, right_id]))
            }
            VecAst::VecMul(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecMul([left_id, right_id]))
            }
            VecAst::VecMinus(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecMinus([left_id, right_id]))
            }
            VecAst::VecDiv(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecDiv([left_id, right_id]))
            }
            VecAst::VecMulSgn(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecMulSgn([left_id, right_id]))
            }
            VecAst::VecSqrtSgn(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::VecSqrtSgn([left_id, right_id]))
            }
            VecAst::Ite(_, _, _) => todo!(),
            VecAst::Sqrt(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::Sqrt([id]))
            }
            VecAst::Sgn(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::Sgn([id]))
            }
            VecAst::Neg(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::Neg([id]))
            }
            VecAst::VecNeg(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecNeg([id]))
            }
            VecAst::VecSqrt(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecSqrt([id]))
            }
            VecAst::VecSgn(inner) => {
                let id = inner.to_recexpr(expr);
                expr.add(VecLang::VecSgn([id]))
            }
            VecAst::VecMAC(a, b, c) => {
                let a_id = a.to_recexpr(expr);
                let b_id = b.to_recexpr(expr);
                let c_id = c.to_recexpr(expr);
                expr.add(VecLang::VecMAC([a_id, b_id, c_id]))
            }
            VecAst::VecMULS(a, b, c) => {
                let a_id = a.to_recexpr(expr);
                let b_id = b.to_recexpr(expr);
                let c_id = c.to_recexpr(expr);
                expr.add(VecLang::VecMULS([a_id, b_id, c_id]))
            }
            VecAst::Let(a, b) => {
                let a_id = expr.add(VecLang::Symbol(egg::Symbol::from(a)));
                let b_id = b.to_recexpr(expr);
                expr.add(VecLang::Let([a_id, b_id]))
            }
            VecAst::List(items) => {
                let ids =
                    items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
                expr.add(VecLang::List(ids.into_boxed_slice()))
            }
            VecAst::Vec(items) => {
                let ids =
                    items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
                expr.add(VecLang::Vec(ids.into_boxed_slice()))
            }
            VecAst::LitVec(items) => {
                let ids =
                    items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
                expr.add(VecLang::LitVec(ids.into_boxed_slice()))
            }
            VecAst::Get(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Get([left_id, right_id]))
            }
            VecAst::Concat(left, right) => {
                let left_id = left.to_recexpr(expr);
                let right_id = right.to_recexpr(expr);
                expr.add(VecLang::Concat([left_id, right_id]))
            }
            VecAst::Const(v) => expr.add(VecLang::Const(v.clone())),
            VecAst::Symbol(s) => expr.add(VecLang::Symbol(s.into())),
        }
    }
}

fn subtree(
    expr: &egg::RecExpr<VecLang>,
    new_root: Id,
) -> egg::RecExpr<VecLang> {
    expr[new_root].build_recexpr(|id| expr[id].clone())
}

impl From<egg::RecExpr<VecLang>> for VecAst {
    fn from(expr: egg::RecExpr<VecLang>) -> Self {
        let root: Id = Id::from(expr.as_ref().len() - 1);
        match &expr[root] {
            VecLang::Add([left, right]) => VecAst::Add(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Mul([left, right]) => VecAst::Mul(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Minus([left, right]) => VecAst::Minus(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Div([left, right]) => VecAst::Div(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Or([left, right]) => VecAst::Or(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::And([left, right]) => VecAst::And(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Ite(_) => todo!(),
            VecLang::Lt([left, right]) => VecAst::Lt(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::SqrtSgn([left, right]) => VecAst::SqrtSgn(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Sgn([inner]) => {
                VecAst::Sgn(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::Sqrt([inner]) => {
                VecAst::Sqrt(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::Neg([inner]) => {
                VecAst::Neg(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::List(_) => todo!(),
            VecLang::Vec(items) => VecAst::Vec(
                items
                    .iter()
                    .map(|id| subtree(&expr, *id).into())
                    .collect_vec(),
            ),
            VecLang::LitVec(items) => VecAst::LitVec(
                items
                    .iter()
                    .map(|id| subtree(&expr, *id).into())
                    .collect_vec(),
            ),
            VecLang::Get([left, right]) => VecAst::Get(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::Concat([left, right]) => VecAst::Concat(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecAdd([left, right]) => VecAst::VecAdd(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMinus([left, right]) => VecAst::VecMinus(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMul([left, right]) => VecAst::VecMul(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecDiv([left, right]) => VecAst::VecDiv(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecMulSgn([left, right]) => VecAst::VecMulSgn(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecSqrtSgn([left, right]) => VecAst::VecSqrtSgn(
                Box::new(subtree(&expr, *left).into()),
                Box::new(subtree(&expr, *right).into()),
            ),
            VecLang::VecNeg([inner]) => {
                VecAst::VecNeg(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::VecSqrt([inner]) => {
                VecAst::VecSqrt(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::VecSgn([inner]) => {
                VecAst::VecSgn(Box::new(subtree(&expr, *inner).into()))
            }
            VecLang::VecMAC([a, b, c]) => VecAst::VecMAC(
                Box::new(subtree(&expr, *a).into()),
                Box::new(subtree(&expr, *b).into()),
                Box::new(subtree(&expr, *c).into()),
            ),
            VecLang::VecMULS([a, b, c]) => VecAst::VecMULS(
                Box::new(subtree(&expr, *a).into()),
                Box::new(subtree(&expr, *b).into()),
                Box::new(subtree(&expr, *c).into()),
            ),
            VecLang::Let([a, b]) => {
                println!("{}", subtree(&expr, *a));
                VecAst::Let(
                    String::from("test"),
                    Box::new(subtree(&expr, *b).into()),
                )
            }
            VecLang::Const(v) => VecAst::Const(v.clone()),
            VecLang::Symbol(sym) => VecAst::Symbol(sym.to_string()),
        }
    }
}

impl From<VecAst> for egg::RecExpr<VecLang> {
    fn from(ast: VecAst) -> Self {
        let mut expr = egg::RecExpr::default();
        ast.to_recexpr(&mut expr);
        expr
    }
}

impl comp_gen::FromPattern for VecLang {
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self> {
        pat.as_ref()
            .iter()
            .map(|node| match node {
                egg::ENodeOrVar::Var(v) => {
                    VecLang::Symbol(v.to_string().into())
                }
                egg::ENodeOrVar::ENode(node) => node.clone(),
            })
            .collect::<Vec<_>>()
            .into()
    }
}

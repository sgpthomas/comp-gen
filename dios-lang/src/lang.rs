use babble::ast_node::{Arity, AstNode, Expr};
use babble::learn::{LibId, ParseLibIdError};
use babble::teachable::{BindingExpr, DeBruijnIndex, Teachable};
use egg::{self, Id};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VecOp {
    // unops
    Sgn,
    VecSgn,
    Sqrt,
    VecSqrt,
    Neg,
    VecNeg,

    // binops
    Add,
    VecAdd,
    Mul,
    VecMul,
    Minus,
    VecMinus,
    Div,
    VecDiv,
    Or,
    And,
    Lt,
    SqrtSgn,
    VecSqrtSgn,
    VecMulSgn,
    Concat,
    Let,
    Get,

    // triops
    VecMAC,
    VecMULS,

    // variable length
    List,
    Vec,
    LitVec,

    // other
    Const(Value),
    Symbol(egg::Symbol),

    // teachable ops
    Lambda,
    Apply,
    LambdaVar(DeBruijnIndex),
    Lib(LibId),
    LibVar(LibId),
}

impl FromStr for VecOp {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "sgn" => Self::Sgn,
            "VecSgn" => Self::VecSgn,
            "sqrt" => Self::Sqrt,
            "VecSqrt" => Self::VecSqrt,
            "neg" => Self::Neg,
            "VecNeg" => Self::VecNeg,

            "+" => Self::Add,
            "VecAdd" => Self::VecAdd,
            "*" => Self::Mul,
            "VecMul" => Self::VecMul,
            "-" => Self::Minus,
            "VecMinus" => Self::VecMinus,
            "/" => Self::Div,
            "VecDiv" => Self::VecDiv,
            "or" => Self::Or,
            "&&" => Self::And,
            "<" => Self::Lt,
            "sqrtsgn" => Self::SqrtSgn,
            "VecSqrtSgn" => Self::VecSqrtSgn,
            "VecMulSgn" => Self::VecMulSgn,
            "Concat" => Self::Concat,
            "let" => Self::Let,
            "Get" => Self::Get,

            "VecMAC" => Self::VecMAC,
            "VecMULS" => Self::VecMULS,

            "List" => Self::List,
            "Vec" => Self::Vec,
            "LitVec" => Self::LitVec,

            "lambda" => Self::Lambda,
            "apply" => Self::Apply,

            input => input
                .parse()
                .map(|x| Self::Const(Value::Int(x)))
                .or_else(|_| input.parse().map(|x| Self::Const(Value::Bool(x))))
                .or_else(|_| input.parse().map(Self::LambdaVar))
                .or_else(|_| input.parse().map(Self::LibVar))
                .or_else(|_| {
                    input
                        .strip_prefix("lib ")
                        .ok_or(ParseLibIdError::NoLeadingL)
                        .and_then(|x| x.parse().map(Self::Lib))
                })
                .unwrap_or_else(|_| Self::Symbol(input.into())),
        })
    }
}

impl Display for VecOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VecOp::Sgn => f.write_str("sgn"),
            VecOp::VecSgn => f.write_str("VecSgn"),
            VecOp::Sqrt => f.write_str("sqrt"),
            VecOp::VecSqrt => f.write_str("VecSqrt"),
            VecOp::Neg => f.write_str("neg"),
            VecOp::VecNeg => f.write_str("VecNeg"),
            VecOp::Add => f.write_str("+"),
            VecOp::VecAdd => f.write_str("VecAdd"),
            VecOp::Mul => f.write_str("*"),
            VecOp::VecMul => f.write_str("VecMul"),
            VecOp::Minus => f.write_str("-"),
            VecOp::VecMinus => f.write_str("VecMinus"),
            VecOp::Div => f.write_str("/"),
            VecOp::VecDiv => f.write_str("VecDiv"),
            VecOp::Or => f.write_str("or"),
            VecOp::And => f.write_str("&&"),
            VecOp::Lt => f.write_str("<"),
            VecOp::SqrtSgn => f.write_str("sqrtsgn"),
            VecOp::VecSqrtSgn => f.write_str("VecSqrtSgn"),
            VecOp::VecMulSgn => f.write_str("VecMulSgn"),
            VecOp::Concat => f.write_str("Concat"),
            VecOp::Let => f.write_str("let"),
            VecOp::Get => f.write_str("Get"),
            VecOp::VecMAC => f.write_str("VecMAC"),
            VecOp::VecMULS => f.write_str("VecMULS"),
            VecOp::List => f.write_str("List"),
            VecOp::Vec => f.write_str("Vec"),
            VecOp::LitVec => f.write_str("LitVec"),
            VecOp::Const(v) => Display::fmt(v, f),
            VecOp::Symbol(sym) => Display::fmt(sym, f),
            VecOp::Lambda => f.write_str("lambda"),
            VecOp::Apply => f.write_str("apply"),
            VecOp::LambdaVar(dix) => Display::fmt(dix, f),
            VecOp::Lib(lib_id) => Display::fmt(lib_id, f),
            VecOp::LibVar(lib_id) => Display::fmt(lib_id, f),
        }
    }
}

// egg::define_language! {
//     #[derive(Serialize, Deserialize)]
//     pub enum VecLang {
//         // Id is a key to identify EClasses within an EGraph, represents
//         // children nodes
//         "+" = Add([egg::Id; 2]),
//         "*" = Mul([egg::Id; 2]),
//         "-" = Minus([egg::Id; 2]),
//         "/" = Div([egg::Id; 2]),

//         "or" = Or([egg::Id; 2]),
//         "&&" = And([egg::Id; 2]),
//         "ite" = Ite([egg::Id; 3]),
//         "<" = Lt([egg::Id; 2]),
//         "sqrtsgn" = SqrtSgn([egg::Id; 2]),

//         "sgn" = Sgn([egg::Id; 1]),
//         "sqrt" = Sqrt([egg::Id; 1]),
//         "neg" = Neg([egg::Id; 1]),

//         // Lists have a variable number of elements
//         "List" = List(Box<[egg::Id]>),

//         // Vectors have width elements
//         "Vec" = Vec(Box<[egg::Id]>),

//         // Vector with all literals
//         "LitVec" = LitVec(Box<[egg::Id]>),

//         "Get" = Get([egg::Id; 2]),

//         // Used for partitioning and recombining lists
//         "Concat" = Concat([egg::Id; 2]),

//         // Vector operations that take 2 vectors of inputs
//         "VecAdd" = VecAdd([egg::Id; 2]),
//         "VecMinus" = VecMinus([egg::Id; 2]),
//         "VecMul" = VecMul([egg::Id; 2]),
//         "VecDiv" = VecDiv([egg::Id; 2]),
//         "VecMulSgn" = VecMulSgn([egg::Id; 2]),
//         "VecSqrtSgn" = VecSqrtSgn([egg::Id; 2]),

//         // Vector operations that take 1 vector of inputs
//         "VecNeg" = VecNeg([egg::Id; 1]),
//         "VecSqrt" = VecSqrt([egg::Id; 1]),
//         "VecSgn" = VecSgn([egg::Id; 1]),
//         // "VecRAdd" = VecRAdd([egg::Id; 1]),

//         // MAC takes 3 lists: acc, v1, v2
//         "VecMAC" = VecMAC([egg::Id; 3]),
//         "VecMULS" = VecMULS([egg::Id; 3]),

//         // let binding
//         "let" = Let([egg::Id; 2]),

//         // constant value
//         Const(Value),

//         // language items are parsed in order, and we want symbol to
//         // be a fallback, so we put it last.
//         // `Symbol` is an egg-provided interned string type
//         Symbol(egg::Symbol),
//     }
// }

impl Arity for VecOp {
    fn min_arity(&self) -> usize {
        match self {
            VecOp::Sgn
            | VecOp::VecSgn
            | VecOp::Sqrt
            | VecOp::VecSqrt
            | VecOp::Neg
            | VecOp::VecNeg => 1,

            VecOp::Add
            | VecOp::VecAdd
            | VecOp::Mul
            | VecOp::VecMul
            | VecOp::Minus
            | VecOp::VecMinus
            | VecOp::Div
            | VecOp::VecDiv
            | VecOp::Or
            | VecOp::And
            | VecOp::Lt
            | VecOp::SqrtSgn
            | VecOp::VecSqrtSgn
            | VecOp::VecMulSgn
            | VecOp::Concat
            | VecOp::Let
            | VecOp::Get => 2,

            VecOp::VecMAC | VecOp::VecMULS => 3,

            VecOp::List => 0,
            VecOp::Vec => 0,
            VecOp::LitVec => 0,

            VecOp::Const(_) | VecOp::Symbol(_) => 0,

            // teaching ops
            VecOp::Lambda => 1,
            VecOp::LambdaVar(_) | VecOp::LibVar(_) => 0,
            VecOp::Apply => 2,
            VecOp::Lib(_) => 2,
        }
    }

    fn max_arity(&self) -> Option<usize> {
        match self {
            VecOp::List | VecOp::Vec | VecOp::LitVec => None,
            _ => Some(self.min_arity()),
        }
    }
}

impl Teachable for VecOp {
    fn from_binding_expr<T>(binding_expr: BindingExpr<T>) -> AstNode<Self, T> {
        match binding_expr {
            BindingExpr::Lambda(body) => AstNode::new(Self::Lambda, [body]),
            BindingExpr::Apply(fun, arg) => {
                AstNode::new(Self::Apply, [fun, arg])
            }
            BindingExpr::Var(index) => AstNode::leaf(Self::LambdaVar(index)),
            BindingExpr::Lib(ix, bound_value, body) => {
                AstNode::new(Self::Lib(ix), [bound_value, body])
            }
            BindingExpr::LibVar(ix) => AstNode::leaf(Self::LibVar(ix)),
        }
    }

    fn as_binding_expr<T>(node: &AstNode<Self, T>) -> Option<BindingExpr<&T>> {
        match node.as_parts() {
            (Self::Lambda, [body]) => Some(BindingExpr::Lambda(body)),
            (Self::Apply, [fun, arg]) => Some(BindingExpr::Apply(fun, arg)),
            (Self::LambdaVar(index), []) => Some(BindingExpr::Var(*index)),
            (Self::Lib(ix), [bound_value, body]) => {
                Some(BindingExpr::Lib(*ix, bound_value, body))
            }
            (Self::LibVar(ix), []) => Some(BindingExpr::LibVar(*ix)),
            _ => None,
        }
    }

    fn list() -> Self {
        Self::List
    }
}

// #[derive(Debug, Clone)]
// pub enum VecAst {
//     Add(Box<VecAst>, Box<VecAst>),
//     Mul(Box<VecAst>, Box<VecAst>),
//     Minus(Box<VecAst>, Box<VecAst>),
//     Div(Box<VecAst>, Box<VecAst>),

//     Or(Box<VecAst>, Box<VecAst>),
//     And(Box<VecAst>, Box<VecAst>),
//     #[allow(unused)]
//     Ite(Box<VecAst>, Box<VecAst>, Box<VecAst>),
//     Lt(Box<VecAst>, Box<VecAst>),
//     SqrtSgn(Box<VecAst>, Box<VecAst>),
//     Get(Box<VecAst>, Box<VecAst>),
//     Concat(Box<VecAst>, Box<VecAst>),

//     Sqrt(Box<VecAst>),
//     Sgn(Box<VecAst>),
//     Neg(Box<VecAst>),

//     List(Vec<VecAst>),
//     Vec(Vec<VecAst>),
//     LitVec(Vec<VecAst>),

//     VecAdd(Box<VecAst>, Box<VecAst>),
//     VecMul(Box<VecAst>, Box<VecAst>),
//     VecMinus(Box<VecAst>, Box<VecAst>),
//     VecDiv(Box<VecAst>, Box<VecAst>),
//     VecMulSgn(Box<VecAst>, Box<VecAst>),
//     VecSqrtSgn(Box<VecAst>, Box<VecAst>),

//     VecNeg(Box<VecAst>),
//     VecSqrt(Box<VecAst>),
//     VecSgn(Box<VecAst>),

//     VecMAC(Box<VecAst>, Box<VecAst>, Box<VecAst>),
//     VecMULS(Box<VecAst>, Box<VecAst>, Box<VecAst>),

//     Let(String, Box<VecAst>),

//     Const(Value),
//     Symbol(String),
// }

// /// A simple visitor for VecAst
// pub trait Visitor {
//     fn do_pass(&mut self, ast: VecAst) -> VecAst {
//         match ast {
//             VecAst::Add(l, r) => self.visit_add(*l, *r),
//             VecAst::Mul(l, r) => self.visit_mul(*l, *r),
//             VecAst::Minus(l, r) => self.visit_minus(*l, *r),
//             VecAst::Div(l, r) => self.visit_div(*l, *r),
//             VecAst::Or(l, r) => self.visit_or(*l, *r),
//             VecAst::And(l, r) => self.visit_and(*l, *r),
//             VecAst::Ite(a, b, c) => self.visit_ite(*a, *b, *c),
//             VecAst::Lt(l, r) => self.visit_lt(*l, *r),
//             VecAst::SqrtSgn(l, r) => self.visit_sqrt_sgn(*l, *r),
//             VecAst::Get(l, r) => self.visit_get(*l, *r),
//             VecAst::Concat(l, r) => self.visit_concat(*l, *r),
//             VecAst::Sqrt(x) => self.visit_sqrt(*x),
//             VecAst::Sgn(x) => self.visit_sgn(*x),
//             VecAst::Neg(x) => self.visit_neg(*x),
//             VecAst::List(x) => self.visit_list(x),
//             VecAst::Vec(x) => self.visit_vec(x),
//             VecAst::LitVec(x) => self.visit_lit_vec(x),
//             VecAst::VecAdd(l, r) => self.visit_vec_add(*l, *r),
//             VecAst::VecMul(l, r) => self.visit_vec_mul(*l, *r),
//             VecAst::VecMinus(l, r) => self.visit_vec_minus(*l, *r),
//             VecAst::VecDiv(l, r) => self.visit_vec_div(*l, *r),
//             VecAst::VecMulSgn(l, r) => self.visit_vec_mulsgn(*l, *r),
//             VecAst::VecSqrtSgn(l, r) => self.visit_vec_sqrtsgn(*l, *r),
//             VecAst::VecNeg(x) => self.visit_vec_neg(*x),
//             VecAst::VecSqrt(x) => self.visit_vec_sqrt(*x),
//             VecAst::VecSgn(x) => self.visit_vec_sgn(*x),
//             VecAst::VecMAC(a, b, c) => self.visit_vec_mac(*a, *b, *c),
//             VecAst::VecMULS(a, b, c) => self.visit_vec_muls(*a, *b, *c),
//             VecAst::Let(l, r) => self.visit_let(l, *r),
//             VecAst::Const(x) => self.visit_const(x),
//             VecAst::Symbol(x) => self.visit_symbol(x),
//         }
//     }

//     fn visit_add(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Add(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_mul(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Mul(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_minus(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Minus(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_div(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Div(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_or(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Or(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_and(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::And(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_ite(&mut self, cond: VecAst, tru: VecAst, fls: VecAst) -> VecAst {
//         VecAst::Ite(
//             Box::new(self.do_pass(cond)),
//             Box::new(self.do_pass(tru)),
//             Box::new(self.do_pass(fls)),
//         )
//     }

//     fn visit_lt(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Lt(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_sqrt_sgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::SqrtSgn(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_get(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Get(Box::new(self.do_pass(left)), Box::new(self.do_pass(right)))
//     }

//     fn visit_concat(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::Concat(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_sqrt(&mut self, inner: VecAst) -> VecAst {
//         VecAst::Sqrt(Box::new(self.do_pass(inner)))
//     }

//     fn visit_sgn(&mut self, inner: VecAst) -> VecAst {
//         VecAst::Sgn(Box::new(self.do_pass(inner)))
//     }

//     fn visit_neg(&mut self, inner: VecAst) -> VecAst {
//         VecAst::Neg(Box::new(self.do_pass(inner)))
//     }

//     fn visit_list(&mut self, elems: Vec<VecAst>) -> VecAst {
//         VecAst::List(elems.into_iter().map(|x| self.do_pass(x)).collect())
//     }

//     fn visit_vec(&mut self, elems: Vec<VecAst>) -> VecAst {
//         VecAst::Vec(elems.into_iter().map(|x| self.do_pass(x)).collect())
//     }

//     fn visit_lit_vec(&mut self, elems: Vec<VecAst>) -> VecAst {
//         VecAst::LitVec(elems.into_iter().map(|x| self.do_pass(x)).collect())
//     }

//     fn visit_vec_add(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecAdd(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_mul(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecMul(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_minus(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecMinus(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_div(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecDiv(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_mulsgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecMulSgn(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_sqrtsgn(&mut self, left: VecAst, right: VecAst) -> VecAst {
//         VecAst::VecSqrtSgn(
//             Box::new(self.do_pass(left)),
//             Box::new(self.do_pass(right)),
//         )
//     }

//     fn visit_vec_neg(&mut self, inner: VecAst) -> VecAst {
//         VecAst::VecNeg(Box::new(self.do_pass(inner)))
//     }

//     fn visit_vec_sqrt(&mut self, inner: VecAst) -> VecAst {
//         VecAst::VecSqrt(Box::new(self.do_pass(inner)))
//     }

//     fn visit_vec_sgn(&mut self, inner: VecAst) -> VecAst {
//         VecAst::VecSgn(Box::new(self.do_pass(inner)))
//     }

//     fn visit_vec_mac(&mut self, a: VecAst, b: VecAst, c: VecAst) -> VecAst {
//         VecAst::VecMAC(
//             Box::new(self.do_pass(a)),
//             Box::new(self.do_pass(b)),
//             Box::new(self.do_pass(c)),
//         )
//     }

//     fn visit_vec_muls(&mut self, a: VecAst, b: VecAst, c: VecAst) -> VecAst {
//         VecAst::VecMULS(
//             Box::new(self.do_pass(a)),
//             Box::new(self.do_pass(b)),
//             Box::new(self.do_pass(c)),
//         )
//     }

//     fn visit_let(&mut self, name: String, val: VecAst) -> VecAst {
//         VecAst::Let(name, Box::new(self.do_pass(val)))
//     }

//     fn visit_const(&mut self, val: Value) -> VecAst {
//         VecAst::Const(val)
//     }

//     fn visit_symbol(&mut self, symbol: String) -> VecAst {
//         VecAst::Symbol(symbol)
//     }
// }

// impl VecAst {
//     fn to_recexpr(&self, expr: &mut egg::RecExpr<VecLang>) -> Id {
//         match self {
//             VecAst::Add(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Add([left_id, right_id]))
//             }
//             VecAst::Mul(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Mul([left_id, right_id]))
//             }
//             VecAst::Minus(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Minus([left_id, right_id]))
//             }
//             VecAst::Div(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Div([left_id, right_id]))
//             }
//             VecAst::Or(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Or([left_id, right_id]))
//             }
//             VecAst::And(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::And([left_id, right_id]))
//             }
//             VecAst::Lt(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Lt([left_id, right_id]))
//             }
//             VecAst::SqrtSgn(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::SqrtSgn([left_id, right_id]))
//             }
//             VecAst::VecAdd(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecAdd([left_id, right_id]))
//             }
//             VecAst::VecMul(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecMul([left_id, right_id]))
//             }
//             VecAst::VecMinus(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecMinus([left_id, right_id]))
//             }
//             VecAst::VecDiv(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecDiv([left_id, right_id]))
//             }
//             VecAst::VecMulSgn(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecMulSgn([left_id, right_id]))
//             }
//             VecAst::VecSqrtSgn(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::VecSqrtSgn([left_id, right_id]))
//             }
//             VecAst::Ite(_, _, _) => todo!(),
//             VecAst::Sqrt(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::Sqrt([id]))
//             }
//             VecAst::Sgn(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::Sgn([id]))
//             }
//             VecAst::Neg(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::Neg([id]))
//             }
//             VecAst::VecNeg(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::VecNeg([id]))
//             }
//             VecAst::VecSqrt(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::VecSqrt([id]))
//             }
//             VecAst::VecSgn(inner) => {
//                 let id = inner.to_recexpr(expr);
//                 expr.add(VecLang::VecSgn([id]))
//             }
//             VecAst::VecMAC(a, b, c) => {
//                 let a_id = a.to_recexpr(expr);
//                 let b_id = b.to_recexpr(expr);
//                 let c_id = c.to_recexpr(expr);
//                 expr.add(VecLang::VecMAC([a_id, b_id, c_id]))
//             }
//             VecAst::VecMULS(a, b, c) => {
//                 let a_id = a.to_recexpr(expr);
//                 let b_id = b.to_recexpr(expr);
//                 let c_id = c.to_recexpr(expr);
//                 expr.add(VecLang::VecMULS([a_id, b_id, c_id]))
//             }
//             VecAst::Let(a, b) => {
//                 let a_id = expr.add(VecLang::Symbol(egg::Symbol::from(a)));
//                 let b_id = b.to_recexpr(expr);
//                 expr.add(VecLang::Let([a_id, b_id]))
//             }
//             VecAst::List(items) => {
//                 let ids =
//                     items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
//                 expr.add(VecLang::List(ids.into_boxed_slice()))
//             }
//             VecAst::Vec(items) => {
//                 let ids =
//                     items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
//                 expr.add(VecLang::Vec(ids.into_boxed_slice()))
//             }
//             VecAst::LitVec(items) => {
//                 let ids =
//                     items.iter().map(|it| it.to_recexpr(expr)).collect_vec();
//                 expr.add(VecLang::LitVec(ids.into_boxed_slice()))
//             }
//             VecAst::Get(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Get([left_id, right_id]))
//             }
//             VecAst::Concat(left, right) => {
//                 let left_id = left.to_recexpr(expr);
//                 let right_id = right.to_recexpr(expr);
//                 expr.add(VecLang::Concat([left_id, right_id]))
//             }
//             VecAst::Const(v) => expr.add(VecLang::Const(v.clone())),
//             VecAst::Symbol(s) => expr.add(VecLang::Symbol(s.into())),
//         }
//     }
// }

// /// Flat representation of the VecLang AST.
// /// This is what lives in the EGraph
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct FlatAst(AstNode<VecOp>);

impl FlatAst {
    #[allow(unused)]
    pub fn new(inner: AstNode<VecOp>) -> Self {
        FlatAst(inner)
    }

    pub fn new_from_parts(op: VecOp, args: Vec<egg::Id>) -> Self {
        FlatAst(AstNode::new(op, args))
    }

    pub fn new_const(value: Value) -> Self {
        FlatAst(AstNode::new(VecOp::Const(value), vec![]))
    }

    pub fn inner(&self) -> &AstNode<VecOp> {
        &self.0
    }

    #[allow(unused)]
    pub fn into_inner(self) -> AstNode<VecOp> {
        self.0
    }
}

impl egg::Language for FlatAst {
    type Discriminant = <AstNode<VecOp> as egg::Language>::Discriminant;

    fn matches(&self, other: &Self) -> bool {
        self.0.matches(&other.0)
    }

    fn children(&self) -> &[Id] {
        self.0.children()
    }

    fn children_mut(&mut self) -> &mut [Id] {
        self.0.children_mut()
    }

    fn discriminant(&self) -> Self::Discriminant {
        self.0.discriminant()
    }
}

impl egg::FromOp for FlatAst {
    type Error = <AstNode<VecOp> as egg::FromOp>::Error;

    fn from_op(op: &str, children: Vec<Id>) -> Result<Self, Self::Error> {
        Ok(FlatAst(AstNode::<VecOp>::from_op(op, children)?))
    }
}

impl Display for FlatAst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Recursive representation of the VecLang AST
/// This is useful for performing tree transformations
#[derive(Clone)]
pub struct RecAst(Expr<VecOp>);

impl RecAst {
    pub fn new(expr: Expr<VecOp>) -> Self {
        RecAst(expr)
    }

    pub fn new_from_parts(op: VecOp, parts: Vec<Expr<VecOp>>) -> Self {
        RecAst(Expr(AstNode::new(op, parts)))
    }

    pub fn into_inner(self) -> Expr<VecOp> {
        self.0
    }

    pub fn into_parts(self) -> (VecOp, Vec<Expr<VecOp>>) {
        self.0.into_inner().into_parts()
    }

    #[allow(unused)]
    pub fn into_args(self) -> Vec<Expr<VecOp>> {
        self.into_parts().1
    }
}

impl From<FlatAst> for AstNode<VecOp> {
    fn from(value: FlatAst) -> Self {
        value.0
    }
}

impl From<Expr<VecOp>> for RecAst {
    fn from(value: Expr<VecOp>) -> Self {
        Self(value)
    }
}

impl From<RecAst> for egg::RecExpr<FlatAst> {
    fn from(value: RecAst) -> Self {
        let rec_expr: egg::RecExpr<AstNode<VecOp>> = value.into_inner().into();
        rec_expr
            .as_ref()
            .into_iter()
            .map(|node| FlatAst::new(node.clone()))
            .collect::<Vec<_>>()
            .into()
    }
}

impl comp_gen::FromPattern for FlatAst {
    /// Convert egg patterns into language expressions by mapping pattern variables
    /// into language variables
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self> {
        pat.as_ref()
            .iter()
            .map(|node| match node {
                egg::ENodeOrVar::Var(v) => FlatAst::new_from_parts(
                    VecOp::Symbol(v.to_string().into()),
                    vec![],
                ),
                egg::ENodeOrVar::ENode(node) => node.clone(),
            })
            .collect::<Vec<_>>()
            .into()
    }
}

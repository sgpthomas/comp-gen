use std::str::FromStr;

use comp_gen::{ruler::egg, FromPattern};
use itertools::Itertools;

use crate::lang;

pub trait Desugar {
    fn desugar(self, n_lanes: usize) -> Self;
}

impl Desugar for egg::Pattern<lang::VecLang> {
    fn desugar(self, n_lanes: usize) -> Self {
        let l: lang::VecAst = lang::VecLang::from_pattern(&self.ast).into();
        let desugared: lang::VecAst = l.desugar(n_lanes);
        let vl: egg::RecExpr<lang::VecLang> = desugared.into();

        // map symbols to vars, everything else to enodes
        let p: egg::RecExpr<egg::ENodeOrVar<lang::VecLang>> = vl
            .as_ref()
            .iter()
            .map(|l: &lang::VecLang| match l {
                lang::VecLang::Symbol(s) => egg::ENodeOrVar::Var(
                    egg::Var::from_str(s.as_str())
                        .expect(&format!("Couldn't varify '{s}' in {self}")),
                ),
                x => egg::ENodeOrVar::ENode(x.clone()),
            })
            .collect_vec()
            .into();

        p.into()
    }
}

impl Desugar for lang::VecAst {
    /// Expand single-lane vector instructions to some number of lanes.
    fn desugar(self, n_lanes: usize) -> Self {
        match self {
            lang::VecAst::Vec(items) if items.len() == 1 => lang::VecAst::Vec(
                (0..n_lanes)
                    .into_iter()
                    .map(|n| items[0].clone().rename(&format!("{n}")))
                    .collect_vec(),
            ),
            x @ lang::VecAst::Vec(_) => {
                panic!("Can't desugar Vecs with more than one lane.\n{:?}", x)
            }
            x @ lang::VecAst::LitVec(_) => {
                panic!(
                    "Can't desugar LitVecs with more than one lane.\n{:?}",
                    x
                )
            }
            lang::VecAst::Add(left, right) => lang::VecAst::Add(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Mul(left, right) => lang::VecAst::Mul(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Minus(left, right) => lang::VecAst::Minus(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Div(left, right) => lang::VecAst::Div(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Or(left, right) => lang::VecAst::Or(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::And(left, right) => lang::VecAst::And(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Ite(_, _, _) => todo!(),
            lang::VecAst::Lt(left, right) => lang::VecAst::Lt(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::Sqrt(inner) => {
                lang::VecAst::Sqrt(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::Sgn(inner) => {
                lang::VecAst::Sgn(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::Neg(inner) => {
                lang::VecAst::Neg(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::VecAdd(left, right) => lang::VecAst::VecAdd(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::VecMul(left, right) => lang::VecAst::VecMul(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::VecMinus(left, right) => lang::VecAst::VecMinus(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::VecDiv(left, right) => lang::VecAst::VecDiv(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::VecNeg(inner) => {
                lang::VecAst::VecNeg(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::VecSqrt(inner) => {
                lang::VecAst::VecSqrt(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::VecSgn(inner) => {
                lang::VecAst::VecSgn(Box::new(inner.desugar(n_lanes)))
            }
            lang::VecAst::VecMAC(a, b, c) => lang::VecAst::VecMAC(
                Box::new(a.desugar(n_lanes)),
                Box::new(b.desugar(n_lanes)),
                Box::new(c.desugar(n_lanes)),
            ),
            lang::VecAst::Get(left, right) => lang::VecAst::Get(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),

            x @ lang::VecAst::Const(_) => x,
            x @ lang::VecAst::Symbol(_) => x,
        }
    }
}

pub trait AlphaRenamable {
    fn rename(self, suffix: &str) -> Self;
}

impl AlphaRenamable for lang::VecAst {
    fn rename(self, suffix: &str) -> Self {
        match self {
            lang::VecAst::Add(x, y) => lang::VecAst::Add(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Mul(x, y) => lang::VecAst::Mul(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Minus(x, y) => lang::VecAst::Minus(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Div(x, y) => lang::VecAst::Div(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Or(x, y) => lang::VecAst::Or(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::And(x, y) => lang::VecAst::And(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Ite(b, t, f) => lang::VecAst::Ite(
                Box::new(b.rename(suffix)),
                Box::new(t.rename(suffix)),
                Box::new(f.rename(suffix)),
            ),
            lang::VecAst::Lt(x, y) => lang::VecAst::Lt(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::Sqrt(x) => {
                lang::VecAst::Sqrt(Box::new(x.rename(suffix)))
            }
            lang::VecAst::Sgn(x) => {
                lang::VecAst::Sgn(Box::new(x.rename(suffix)))
            }
            lang::VecAst::Neg(x) => {
                lang::VecAst::Neg(Box::new(x.rename(suffix)))
            }
            lang::VecAst::Vec(items) => lang::VecAst::Vec(
                items.into_iter().map(|x| x.rename(suffix)).collect_vec(),
            ),
            lang::VecAst::LitVec(items) => lang::VecAst::LitVec(
                items.into_iter().map(|x| x.rename(suffix)).collect_vec(),
            ),
            lang::VecAst::Get(x, y) => lang::VecAst::Get(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::VecAdd(x, y) => lang::VecAst::VecAdd(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::VecMul(x, y) => lang::VecAst::VecMul(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::VecMinus(x, y) => lang::VecAst::VecMinus(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::VecDiv(x, y) => lang::VecAst::VecDiv(
                Box::new(x.rename(suffix)),
                Box::new(y.rename(suffix)),
            ),
            lang::VecAst::VecNeg(x) => {
                lang::VecAst::VecNeg(Box::new(x.rename(suffix)))
            }
            lang::VecAst::VecSqrt(x) => {
                lang::VecAst::VecSqrt(Box::new(x.rename(suffix)))
            }
            lang::VecAst::VecSgn(x) => {
                lang::VecAst::VecSgn(Box::new(x.rename(suffix)))
            }
            lang::VecAst::VecMAC(a, b, c) => lang::VecAst::VecMAC(
                Box::new(a.rename(suffix)),
                Box::new(b.rename(suffix)),
                Box::new(c.rename(suffix)),
            ),
            x @ lang::VecAst::Const(_) => x,
            lang::VecAst::Symbol(n) => {
                lang::VecAst::Symbol(format!("{n}{suffix}"))
            }
        }
    }
}

use std::str::FromStr;

use comp_gen::{ruler::egg, FromPattern};
use itertools::Itertools;

use crate::{alpha_rename::AlphaRenamable, lang};

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
            lang::VecAst::List(l) => lang::VecAst::List(
                l.into_iter().map(|i| i.desugar(n_lanes)).collect(),
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
            lang::VecAst::SqrtSgn(left, right) => lang::VecAst::SqrtSgn(
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
            lang::VecAst::VecMulSgn(left, right) => lang::VecAst::VecMulSgn(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
            ),
            lang::VecAst::VecSqrtSgn(left, right) => lang::VecAst::VecSqrtSgn(
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
            lang::VecAst::VecMULS(a, b, c) => lang::VecAst::VecMULS(
                Box::new(a.desugar(n_lanes)),
                Box::new(b.desugar(n_lanes)),
                Box::new(c.desugar(n_lanes)),
            ),
            lang::VecAst::Let(a, b) => {
                lang::VecAst::Let(a, Box::new(b.desugar(n_lanes)))
            }
            lang::VecAst::Concat(left, right) => lang::VecAst::Concat(
                Box::new(left.desugar(n_lanes)),
                Box::new(right.desugar(n_lanes)),
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

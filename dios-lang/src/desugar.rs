use std::str::FromStr;

use crate::{
    alpha_rename::AlphaRenamable,
    lang::{self, FlatAst},
};
use babble::ast_node::{AstNode, Expr};
use comp_gen::FromPattern;
use egg;

pub trait Desugar {
    fn desugar(self, n_lanes: usize) -> Self;
}

impl Desugar for egg::Pattern<lang::FlatAst> {
    /// Extend single-lane Vec patterns into `n_lanes` vector patterns.
    /// For example,
    ///   (Vec (+ ?a ?b)) => (VecAdd (Vec ?a) (Vec ?b))
    /// is extended to (with `n_lanes == 2`):
    ///   (Vec (+ ?a0 ?b0) (+ ?a1 ?b1)) => (VecAdd (Vec ?a0 ?a1) (Vec ?b0 ?b1))
    fn desugar(self, n_lanes: usize) -> Self {
        // start by translating pattern variables into lang::VecOp variables
        let rec_expr: egg::RecExpr<lang::FlatAst> =
            lang::FlatAst::from_pattern(&self.ast);

        // lang::FlatAst is a newtype wrapper over AstNode<lang::VecOp>
        // so we need to map over the rec_expr, calling `.inner()` on
        // all children
        let fixed_rec_expr: egg::RecExpr<AstNode<lang::VecOp>> = rec_expr
            .as_ref()
            .into_iter()
            .map(|fa| fa.inner())
            .cloned()
            .collect::<Vec<_>>()
            .into();

        // translate to a babble::Expr which turns the flat ast into a recursive one
        let expr = Expr::from(fixed_rec_expr);

        // move into our local wrapper over Expr, and then call desugar on it
        let rec_ast = lang::RecAst::from(expr).desugar(n_lanes);

        // translate back into a recexpr
        let desugared_expr: egg::RecExpr<AstNode<lang::VecOp>> =
            rec_ast.into_inner().into();

        // convert back to a pattern ast
        let x: egg::PatternAst<_> = desugared_expr
            .as_ref()
            .iter()
            .map(|node| match node.clone().into_parts() {
                (lang::VecOp::Symbol(s), _) => egg::ENodeOrVar::Var(
                    egg::Var::from_str(s.as_str()).unwrap(),
                ),
                (op, args) => {
                    egg::ENodeOrVar::ENode(FlatAst::new_from_parts(op, args))
                }
            })
            .collect::<Vec<_>>()
            .into();

        // and finally back to a pattern
        x.into()
    }
}

impl Desugar for lang::RecAst {
    fn desugar(self, n_lanes: usize) -> Self {
        let (op, args) = self.into_parts();

        // expand LitVec and Vec constructs to be `n_lanes` by renaming
        // variables to be suffixed with their lane number
        // for anything else, we recursively call `.desguar(n_lanes)` on their children
        let new_args = match op {
            lang::VecOp::LitVec | lang::VecOp::Vec => {
                if args.len() == 1 {
                    (0..n_lanes)
                        .into_iter()
                        .map(|n| args[0].clone().rename(&format!("{n}")))
                        .collect()
                } else {
                    panic!("Don't support desugaring rules with >1 lanes")
                }
            }

            _ => args
                .into_iter()
                .map(|it| lang::RecAst::new(it).desugar(n_lanes))
                .map(|rec_ast| rec_ast.into_inner())
                .collect(),
        };

        // return the new expression
        lang::RecAst::new_from_parts(op, new_args)
    }
}

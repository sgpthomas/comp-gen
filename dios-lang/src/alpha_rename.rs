use babble::ast_node::{AstNode, Expr};

use crate::lang;
// struct AlphaRename<'a> {
//     suffix: &'a str,
// }

// impl<'a> Visitor for AlphaRename<'a> {
//     fn visit_symbol(&mut self, symbol: String) -> lang::VecAst {
//         lang::VecAst::Symbol(format!("{symbol}{}", self.suffix))
//     }
// }

pub trait AlphaRenamable {
    fn rename(self, suffix: &str) -> Self;
}

impl AlphaRenamable for Expr<lang::VecOp> {
    fn rename(self, suffix: &str) -> Self {
        // (AlphaRename { suffix }).do_pass(self)
        let (op, args) = self.into_inner().into_parts();

        // rename all symbols
        let new_op = match op {
            lang::VecOp::Symbol(sym) => {
                lang::VecOp::Symbol(format!("{sym}{suffix}").into())
            }
            x => x,
        };

        // recursive through all arguments
        let new_args = args.into_iter().map(|expr| expr.rename(suffix));

        // reconstruct the new node
        Expr(AstNode::new(new_op, new_args))
    }
}

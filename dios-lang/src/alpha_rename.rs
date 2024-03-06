use crate::lang::{self, Visitor};

struct AlphaRename<'a> {
    suffix: &'a str,
}

impl<'a> Visitor for AlphaRename<'a> {
    fn visit_symbol(&mut self, symbol: String) -> lang::VecAst {
        lang::VecAst::Symbol(format!("{symbol}{}", self.suffix))
    }
}

pub trait AlphaRenamable {
    fn rename(self, suffix: &str) -> Self;
}

impl AlphaRenamable for lang::VecAst {
    fn rename(self, suffix: &str) -> Self {
        (AlphaRename { suffix }).do_pass(self)
    }
}

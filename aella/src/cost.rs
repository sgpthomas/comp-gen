use comp_gen::{
    ruler::egg::{self, CostFunction, Language},
    CostMetrics, FromPattern,
};

use crate::lang;

#[derive(Clone, Debug)]
pub struct AellaCost {
    literal: f64,
    op: f64,
    command: f64,
    asm_instr: f64,
}

impl Default for AellaCost {
    fn default() -> Self {
        Self {
            literal: 0.001,
            op: 2.,
            command: 2.,
            asm_instr: 0.01,
        }
    }
}

impl egg::CostFunction<lang::Aella> for AellaCost {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &lang::Aella, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        let op_cost = match enode {
            // normal operaters
            lang::Aella::Plus(_)
            | lang::Aella::Sub(_)
            | lang::Aella::Times(_)
            | lang::Aella::Div(_)
            | lang::Aella::Eq(_)
            | lang::Aella::Lt(_)
            | lang::Aella::Not(_)
            | lang::Aella::And(_) => self.op,
            // normal commands
            lang::Aella::Seq(_)
            | lang::Aella::While(_)
            | lang::Aella::Assign(_) => self.command,
            // assembly instructions (2 args)
            lang::Aella::AsmMov(_vals) => {
                self.asm_instr
                // if vals.iter().any(|&x| costs(x) > 2. * self.literal) {
                //     self.command
                // } else {
                // }
            }
            // assembly instructions (3 args)
            lang::Aella::AsmAdd(vals)
            | lang::Aella::AsmSub(vals)
            | lang::Aella::AsmSmull(vals)
            | lang::Aella::AsmSdiv(vals) => {
                if vals.iter().any(|&x| costs(x) > self.literal) {
                    self.command
                } else {
                    self.asm_instr
                }
            }
            // base items (literals)
            lang::Aella::Num(..) => self.literal,
            lang::Aella::Var(..) => self.literal,
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

impl CostMetrics<lang::Aella, ()> for AellaCost {
    fn cost_differential(
        &mut self,
        r: &egg::Rewrite<lang::Aella, ()>,
    ) -> Self::Cost {
        if let (Some(lhs), Some(rhs)) =
            (r.searcher.get_pattern_ast(), r.applier.get_pattern_ast())
        {
            let lexp: egg::RecExpr<_> = lang::Aella::from_pattern(lhs);
            let rexp: egg::RecExpr<_> = lang::Aella::from_pattern(rhs);
            let lcost = self.cost_rec(&lexp);
            let rcost = self.cost_rec(&rexp);
            lcost - rcost
        } else {
            todo!()
        }
    }

    fn cost_average(
        &mut self,
        r: &egg::Rewrite<lang::Aella, ()>,
    ) -> Self::Cost {
        if let (Some(lhs), Some(rhs)) =
            (r.searcher.get_pattern_ast(), r.applier.get_pattern_ast())
        {
            let lexp: egg::RecExpr<_> = lang::Aella::from_pattern(lhs);
            let rexp: egg::RecExpr<_> = lang::Aella::from_pattern(rhs);
            let lcost = self.cost_rec(&lexp);
            let rcost = self.cost_rec(&rexp);
            (lcost + rcost) / 2.
        } else {
            todo!()
        }
    }
}

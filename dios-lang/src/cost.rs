use comp_gen::ruler::egg::{self, CostFunction, Language};
use comp_gen::FromPattern;

use crate::lang::VecLang;

pub type DiosRwrite = egg::Rewrite<VecLang, ()>;

#[derive(Clone, Debug)]
pub struct VecCostFn {
    original: bool,
}

impl VecCostFn {
    pub fn original() -> Self {
        VecCostFn { original: true }
    }

    pub fn alternative() -> Self {
        VecCostFn { original: false }
    }
}

impl egg::CostFunction<VecLang> for VecCostFn {
    type Cost = f64;
    // you're passed in an enode whose children are costs instead of eclass ids
    fn cost<C>(&mut self, enode: &VecLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        const LITERAL: f64 = 0.001;
        const STRUCTURE: f64 = 0.1;
        const VEC_OP: f64 = 1.;
        let op_cost: f64 = if self.original {
            2. // original
        } else {
            10.
        };
        const BIG: f64 = 100.0;
        let op_cost = match enode {
            // You get literals for extremely cheap
            VecLang::Const(..) => LITERAL,
            VecLang::Symbol(..) => LITERAL,
            VecLang::Get(..) => LITERAL,

            // And list structures for quite cheap
            VecLang::List(..) => STRUCTURE,
            VecLang::Concat(..) => STRUCTURE,

            // Vectors are cheap if they have literal values
            VecLang::Vec(vals) => {
                // For now, workaround to determine if children are num, symbol,
                // or get
                let non_literals =
                    vals.iter().any(|&x| costs(x) > 3. * LITERAL);
                if non_literals {
                    if self.original {
                        BIG
                    } else {
                        BIG * vals.iter().fold(0., |acc, e| costs(*e) + acc)
                    }
                } else {
                    STRUCTURE
                }
            }
            VecLang::LitVec(..) => LITERAL,

            // But scalar and vector ops cost something
            VecLang::Add(vals) => op_cost * (vals.len() as f64 - 1.),
            VecLang::Mul(vals) => op_cost * (vals.len() as f64 - 1.),
            VecLang::Minus(vals) => op_cost * (vals.len() as f64 - 1.),
            VecLang::Div(vals) => op_cost * (vals.len() as f64 - 1.),

            VecLang::Sgn(..) => op_cost,
            VecLang::Neg(..) => op_cost,
            VecLang::Sqrt(..) => op_cost,

            VecLang::VecAdd(..) => VEC_OP,
            VecLang::VecMinus(..) => VEC_OP,
            VecLang::VecMul(..) => VEC_OP,
            VecLang::VecMAC(..) => VEC_OP,
            VecLang::VecDiv(..) => VEC_OP,
            VecLang::VecNeg(..) => VEC_OP,
            VecLang::VecSqrt(..) => VEC_OP,
            VecLang::VecSgn(..) => VEC_OP,
            VecLang::Or(_) => VEC_OP,
            VecLang::And(_) => VEC_OP,
            VecLang::Ite(_) => VEC_OP,
            VecLang::Lt(_) => VEC_OP,
            VecLang::VecMulSgn(_) => VEC_OP,
            // _ => VEC_OP,
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

impl comp_gen::CostMetrics<VecLang, ()> for VecCostFn {
    fn cost_differential(&mut self, r: &DiosRwrite) -> f64 {
        if let (Some(lhs), Some(rhs)) =
            (r.searcher.get_pattern_ast(), r.applier.get_pattern_ast())
        {
            let lexp: egg::RecExpr<VecLang> = VecLang::from_pattern(lhs);
            let rexp: egg::RecExpr<VecLang> = VecLang::from_pattern(rhs);
            // let mut costfn = VecCostFn {};
            self.cost_rec(&lexp) - self.cost_rec(&rexp)
        } else {
            match r.name.as_str() {
                // "litvec" => 0.099,
                // "+_binop_or_zero_vec" => 102.8,
                // "*_binop_or_zero_vec" => 102.8,
                // "-_binop_or_zero_vec" => 102.8,
                // "vec-mac" => 106.7,
                _ => panic!("rule: {:?}", r),
            }
        }
    }

    fn cost_average(&mut self, r: &DiosRwrite) -> f64 {
        if let (Some(lhs), Some(rhs)) =
            (r.searcher.get_pattern_ast(), r.applier.get_pattern_ast())
        {
            let lexp: egg::RecExpr<VecLang> = VecLang::from_pattern(lhs);
            let rexp: egg::RecExpr<VecLang> = VecLang::from_pattern(rhs);
            // let mut costfn = VecCostFn {};
            (self.cost_rec(&lexp) + self.cost_rec(&rexp)) / 2.
        } else {
            match r.name.as_str() {
                // "litvec" => 100.,
                // "+_binop_or_zero_vec" => 50.,
                // "*_binop_or_zero_vec" => 50.,
                // "-_binop_or_zero_vec" => 50.,
                // "vec-mac" => 100.,
                _ => panic!("rule: {:?}", r),
            }
        }
    }
}

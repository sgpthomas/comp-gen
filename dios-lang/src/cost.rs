use comp_gen::FromPattern;
use egg::{self, CostFunction, Language};
use itertools::Itertools;

use crate::lang::{FlatAst, VecOp};

pub type DiosRwrite = egg::Rewrite<FlatAst, ()>;

#[derive(Clone, Debug)]
pub struct VecCostFn {
    literal: f64,
    structure: f64,
    vec: f64,
    op: f64,
    op_proportional: bool,
    vec_op: f64,
    vec_proportional: bool,
    vec_return_early: bool,
    vec_exact_literal_match: bool,
}

impl VecCostFn {
    pub fn dios() -> Self {
        VecCostFn {
            literal: 0.001,
            structure: 0.1,
            vec: 100.0,
            op: 2.,
            op_proportional: true,
            vec_op: 1.,
            vec_proportional: false,
            vec_return_early: false,
            vec_exact_literal_match: false,
        }
    }

    pub fn alternative() -> Self {
        VecCostFn {
            literal: 0.001,
            structure: 0.1,
            vec: 100.0,
            op: 10.,
            op_proportional: true,
            vec_op: 1.,
            vec_proportional: true,
            vec_return_early: false,
            vec_exact_literal_match: false,
        }
    }

    pub fn accurate() -> Self {
        // semi accurate
        VecCostFn {
            literal: 1.,
            structure: 1.,
            vec: 1.,
            op: 1.,
            op_proportional: false,
            vec_op: 1.,
            vec_proportional: false,
            vec_return_early: true,
            vec_exact_literal_match: true,
        }
    }
}

impl egg::CostFunction<FlatAst> for VecCostFn {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &FlatAst, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        let inner = enode.inner();
        let (op, vals) = inner.clone().into_parts();
        let op_cost = match op {
            // you get literals for extremely cheap
            VecOp::Const(_) | VecOp::Symbol(_) | VecOp::Get => self.literal,

            // and list structures for quite cheap
            VecOp::List | VecOp::Concat => self.structure,

            // vectors are cheap if they have literal values
            VecOp::Vec => {
                // For now, workaround to determine if children are literals
                let non_literals = if self.vec_exact_literal_match {
                    vals.iter().any(|&x| costs(x) != self.literal)
                } else {
                    vals.iter().any(|&x| costs(x) > 3.0 * self.literal)
                };
                if non_literals {
                    if self.vec_proportional {
                        self.vec
                            * vals.iter().fold(0.0, |acc, e| costs(*e) + acc)
                    } else {
                        self.vec
                    }
                } else {
                    let cost =
                        self.structure * (vals.iter().unique().count() as f64);
                    // we don't want to count the cost of loading the literals seaprately
                    // the cost of the vec is the cost of the whole thing
                    if self.vec_return_early {
                        return cost + 1.0;
                    }
                    cost
                }
            }
            VecOp::LitVec => {
                if self.vec_return_early {
                    return self.literal;
                }
                self.literal
            }

            // but scalar and vector ops cost something
            VecOp::Sgn
            | VecOp::Sqrt
            | VecOp::Neg
            | VecOp::Add
            | VecOp::Mul
            | VecOp::Minus
            | VecOp::Div
            | VecOp::Or
            | VecOp::And
            | VecOp::Lt => {
                self.op
                    * (if self.op_proportional {
                        vals.len() as f64 - 1.0
                    } else {
                        1.0
                    })
            }

            VecOp::VecSgn
            | VecOp::VecSqrt
            | VecOp::VecNeg
            | VecOp::VecAdd
            | VecOp::VecMul
            | VecOp::VecMinus
            | VecOp::VecDiv
            | VecOp::SqrtSgn
            | VecOp::VecSqrtSgn
            | VecOp::VecMulSgn
            | VecOp::VecMAC
            | VecOp::VecMULS => self.vec_op,

            VecOp::Let
            | VecOp::Lambda
            | VecOp::Apply
            | VecOp::LambdaVar(_)
            | VecOp::Lib(_)
            | VecOp::LibVar(_) => 0.0,
        };
        inner.fold(op_cost, |sum, id| sum + costs(id))
    }
}

impl comp_gen::CostMetrics<FlatAst, ()> for VecCostFn {
    fn cost_differential(&mut self, r: &DiosRwrite) -> f64 {
        if let (Some(lhs), Some(rhs)) =
            (r.searcher.get_pattern_ast(), r.applier.get_pattern_ast())
        {
            let lexp: egg::RecExpr<FlatAst> = FlatAst::from_pattern(lhs);
            let rexp: egg::RecExpr<FlatAst> = FlatAst::from_pattern(rhs);
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
            let lexp: egg::RecExpr<FlatAst> = FlatAst::from_pattern(lhs);
            let rexp: egg::RecExpr<FlatAst> = FlatAst::from_pattern(rhs);
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

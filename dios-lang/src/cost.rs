use comp_gen::ruler::egg::{self, CostFunction, Language};
use comp_gen::FromPattern;
use itertools::Itertools;

use crate::lang::VecLang;

pub type DiosRwrite = egg::Rewrite<VecLang, ()>;

#[derive(Clone, Debug)]
pub struct VecCostFn {
    literal: f64,
    structure: f64,
    vec: f64,
    op: f64,
    op_proportional: bool,
    vec_op: f64,
    vec_proportional: bool,
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
        }
    }

    pub fn accurate() -> Self {
        // semi accurate
        VecCostFn {
            literal: 1.,
            structure: 1.,
            vec: 1.,
            op: 2.,
            op_proportional: false,
            vec_op: 1.,
            vec_proportional: false,
        }
    }
}

impl egg::CostFunction<VecLang> for VecCostFn {
    type Cost = f64;
    // you're passed in an enode whose children are costs instead of eclass ids
    fn cost<C>(&mut self, enode: &VecLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        // const LITERAL: f64 = 0.001;
        // const STRUCTURE: f64 = 0.1;
        // const VEC_OP: f64 = 1.;
        // let op_cost: f64 = if self.original {
        //     2. // original
        // } else {
        //     10.
        // };
        // const BIG: f64 = 100.0;
        let op_cost = match enode {
            // You get literals for extremely cheap
            VecLang::Const(..) => self.literal,
            VecLang::Symbol(..) => self.literal,
            VecLang::Get(..) => self.literal,

            // And list structures for quite cheap
            VecLang::List(..) => self.structure,
            VecLang::Concat(..) => self.structure,

            // Vectors are cheap if they have literal values
            VecLang::Vec(vals) => {
                // For now, workaround to determine if children are num, symbol,
                // or get
                let non_literals =
                    vals.iter().any(|&x| costs(x) > 3. * self.literal);
                if non_literals {
                    if self.vec_proportional {
                        self.vec
                            * vals.iter().fold(0., |acc, e| costs(*e) + acc)
                    } else {
                        self.vec
                    }
                } else {
                    self.structure * (vals.iter().unique().count() as f64)
                }
            }
            VecLang::LitVec(..) => self.literal,

            // But scalar and vector ops cost something
            VecLang::Add(vals) => {
                self.op
                    * (if self.op_proportional {
                        vals.len() as f64 - 1.
                    } else {
                        1.
                    })
            }
            VecLang::Mul(vals) => {
                self.op
                    * (if self.op_proportional {
                        vals.len() as f64 - 1.
                    } else {
                        1.
                    })
            }
            VecLang::Minus(vals) => {
                self.op
                    * (if self.op_proportional {
                        vals.len() as f64 - 1.
                    } else {
                        1.
                    })
            }
            VecLang::Div(vals) => {
                self.op
                    * (if self.op_proportional {
                        vals.len() as f64 - 1.
                    } else {
                        1.
                    })
            }

            VecLang::Sgn(..) => self.op,
            VecLang::Neg(..) => self.op,
            VecLang::Sqrt(..) => self.op,

            VecLang::VecAdd(..) => self.vec_op,
            VecLang::VecMinus(..) => self.vec_op,
            VecLang::VecMul(..) => self.vec_op,
            VecLang::VecMAC(..) => self.vec_op,
            VecLang::VecDiv(..) => self.vec_op,
            VecLang::VecNeg(..) => self.vec_op,
            VecLang::VecSqrt(..) => self.vec_op,
            VecLang::VecSgn(..) => self.vec_op,
            VecLang::Or(_) => self.vec_op,
            VecLang::And(_) => self.vec_op,
            VecLang::Ite(_) => self.vec_op,
            VecLang::Lt(_) => self.vec_op,
            VecLang::VecMulSgn(_) => self.vec_op,
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

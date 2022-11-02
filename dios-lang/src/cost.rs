use comp_gen::ruler::egg::{self, CostFunction, Language};
use comp_gen::FromPattern;

use crate::{lang::VecLang, recexpr_helpers};

pub type DiosRwrite = egg::Rewrite<VecLang, ()>;

#[derive(Default, Clone)]
pub struct VecCostFn;

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
        const OP: f64 = 2.;
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
                    BIG
                } else {
                    STRUCTURE
                }
            }
            // VecLang::LitVec(..) => LITERAL,

            // But scalar and vector ops cost something
            VecLang::Add(vals) => OP * (vals.len() as f64 - 1.),
            VecLang::Mul(vals) => OP * (vals.len() as f64 - 1.),
            VecLang::Minus(vals) => OP * (vals.len() as f64 - 1.),
            VecLang::Div(vals) => OP * (vals.len() as f64 - 1.),

            VecLang::Sgn(..) => OP,
            VecLang::Neg(..) => OP,
            VecLang::Sqrt(..) => OP,

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
            let mut costfn = VecCostFn {};
            costfn.cost_rec(&lexp) - costfn.cost_rec(&rexp)
        } else {
            match r.name.as_str() {
                "litvec" => 0.099,
                "+_binop_or_zero_vec" => 102.8,
                "*_binop_or_zero_vec" => 102.8,
                "-_binop_or_zero_vec" => 102.8,
                "vec-mac" => 106.7,
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
            let mut costfn = VecCostFn {};
            (costfn.cost_rec(&lexp) + costfn.cost_rec(&rexp)) / 2.
        } else {
            match r.name.as_str() {
                "litvec" => 100.,
                "+_binop_or_zero_vec" => 50.,
                "*_binop_or_zero_vec" => 50.,
                "-_binop_or_zero_vec" => 50.,
                "vec-mac" => 100.,
                _ => panic!("rule: {:?}", r),
            }
        }
    }
}

/// Checks if the searcher `lhs` matches `expr`.
fn match_recexpr(
    pattern: &egg::RecExpr<egg::ENodeOrVar<VecLang>>,
    pattern_root: &egg::ENodeOrVar<VecLang>,
    expr: &egg::RecExpr<VecLang>,
    expr_root: &VecLang,
) -> bool {
    match (pattern_root, expr_root) {
        // no children
        (egg::ENodeOrVar::ENode(VecLang::Const(n0)), VecLang::Const(n1)) => {
            n0 == n1
        }
        (egg::ENodeOrVar::ENode(VecLang::Symbol(s0)), VecLang::Symbol(s1)) => {
            s0 == s1
        }

        // 1 child
        (egg::ENodeOrVar::ENode(VecLang::Sgn(lefts)), VecLang::Sgn(rights))
        | (
            egg::ENodeOrVar::ENode(VecLang::Sqrt(lefts)),
            VecLang::Sqrt(rights),
        )
        | (egg::ENodeOrVar::ENode(VecLang::Neg(lefts)), VecLang::Neg(rights))
        | (
            egg::ENodeOrVar::ENode(VecLang::VecNeg(lefts)),
            VecLang::VecNeg(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecSqrt(lefts)),
            VecLang::VecSqrt(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecSgn(lefts)),
            VecLang::VecSgn(rights),
        ) => lefts.iter().zip(rights.iter()).all(|(l, r)| {
            match_recexpr(pattern, &pattern[*l], expr, &expr[*r])
        }),

        // 2 children
        (egg::ENodeOrVar::ENode(VecLang::Add(lefts)), VecLang::Add(rights))
        | (egg::ENodeOrVar::ENode(VecLang::Mul(lefts)), VecLang::Mul(rights))
        | (
            egg::ENodeOrVar::ENode(VecLang::Minus(lefts)),
            VecLang::Minus(rights),
        )
        | (egg::ENodeOrVar::ENode(VecLang::Div(lefts)), VecLang::Div(rights))
        | (egg::ENodeOrVar::ENode(VecLang::Or(lefts)), VecLang::Or(rights))
        | (egg::ENodeOrVar::ENode(VecLang::And(lefts)), VecLang::And(rights))
        | (egg::ENodeOrVar::ENode(VecLang::Lt(lefts)), VecLang::Lt(rights))
        | (egg::ENodeOrVar::ENode(VecLang::Get(lefts)), VecLang::Get(rights))
        | (
            egg::ENodeOrVar::ENode(VecLang::Concat(lefts)),
            VecLang::Concat(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecAdd(lefts)),
            VecLang::VecAdd(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecMinus(lefts)),
            VecLang::VecMinus(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecMul(lefts)),
            VecLang::VecMul(rights),
        )
        | (
            egg::ENodeOrVar::ENode(VecLang::VecDiv(lefts)),
            VecLang::VecDiv(rights),
        ) => lefts.iter().zip(rights.iter()).all(|(l, r)| {
            match_recexpr(pattern, &pattern[*l], expr, &expr[*r])
        }),

        // 3 children
        (egg::ENodeOrVar::ENode(VecLang::Ite(lefts)), VecLang::Ite(rights))
        | (
            egg::ENodeOrVar::ENode(VecLang::VecMAC(lefts)),
            VecLang::VecMAC(rights),
        ) => lefts.iter().zip(rights.iter()).all(|(l, r)| {
            match_recexpr(pattern, &pattern[*l], expr, &expr[*r])
        }),

        // n childen
        (
            egg::ENodeOrVar::ENode(VecLang::List(lefts)),
            VecLang::List(rights),
        )
        | (egg::ENodeOrVar::ENode(VecLang::Vec(lefts)), VecLang::Vec(rights))
        // | (
        //     egg::ENodeOrVar::ENode(VecLang::LitVec(lefts)),
        //     VecLang::LitVec(rights),
        // )
	    => lefts.iter().zip(rights.iter()).all(|(l, r)| {
            match_recexpr(pattern, &pattern[*l], expr, &expr[*r])
        }),

        // else, we checked everything, return false
        (egg::ENodeOrVar::ENode(_), _) => false,

        // if the pattern is a variable, it matches anything
        (egg::ENodeOrVar::Var(_), _) => true,
    }
}

#[allow(unused)]
fn match_pattern_against_egraph<A: egg::Analysis<VecLang>>(
    pattern: &egg::RecExpr<egg::ENodeOrVar<VecLang>>,
    pattern_id: &egg::ENodeOrVar<VecLang>,
    egraph: &egg::EGraph<VecLang, A>,
    expr: &VecLang,
) -> usize {
    // match on expr & pattern, if they have the same top-level, check the number of children
    // that match this?

    // egraph[id]
    //     .nodes
    //     .iter()
    //     .map(|enode| if matches(pattern, enode) { 1 } else { 0 })
    //     .sum()
    todo!()
}

/// Defines a cost function based on the rules in a phase.
/// Roughly, this looks at the LHS of each rule in a phase,
/// and gives expressions that match some LHS a low cost.
pub struct PhaseCostFn {
    rules: Vec<DiosRwrite>,
    expr: egg::RecExpr<VecLang>,
}

impl PhaseCostFn {
    #[allow(unused)]
    pub fn from_rules(
        rules: Vec<DiosRwrite>,
        expr: egg::RecExpr<VecLang>,
    ) -> Self {
        for r in &rules {
            if let Some(r) = r.searcher.get_pattern_ast() {
                eprintln!("lhs: {}", r.pretty(80));
            }
        }
        PhaseCostFn { rules, expr }
    }
}

impl egg::CostFunction<VecLang> for PhaseCostFn {
    type Cost = f64;

    fn cost<C>(&mut self, enode: &VecLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(egg::Id) -> Self::Cost,
    {
        const _BIG: f64 = 100.0;

        let expr: egg::RecExpr<VecLang> =
            enode.build_recexpr(|id| self.expr[id].clone());
        let raw_this_cost: f64 = self
            .rules
            .iter()
            .flat_map(|rw| rw.searcher.get_pattern_ast())
            .fold(0, |acc, it| {
                if match_recexpr(
                    it,
                    recexpr_helpers::root(it),
                    &expr,
                    recexpr_helpers::root(&expr),
                ) {
                    acc + 1
                } else {
                    acc
                }
            }) as f64;

        let raw_tot_cost = raw_this_cost
            + match enode {
                VecLang::Const(_) | VecLang::Symbol(_) => 0.0,
                // 1 children
                VecLang::Sgn(ids)
                | VecLang::Sqrt(ids)
                | VecLang::Neg(ids)
                | VecLang::VecNeg(ids)
                | VecLang::VecSqrt(ids)
                | VecLang::VecSgn(ids) => {
                    ids.iter().fold(0.0, |acc, it| acc + costs(*it))
                }

                // 2 children
                VecLang::Add(ids)
                    | VecLang::Mul(ids)
                    | VecLang::Minus(ids)
                    | VecLang::Div(ids)
                    | VecLang::Or(ids)
                    | VecLang::And(ids)
                    | VecLang::Lt(ids)
                    | VecLang::Get(ids)
                    | VecLang::Concat(ids)
                    | VecLang::VecAdd(ids)
                    | VecLang::VecMinus(ids)
                    | VecLang::VecMul(ids)
		    | VecLang::VecMulSgn(ids)
                    | VecLang::VecDiv(ids) => {
			ids.iter().fold(0.0, |acc, it| acc + costs(*it))
                    }

                // 3 children
                VecLang::VecMAC(ids) | VecLang::Ite(ids) => {
                    ids.iter().fold(0.0, |acc, it| acc + costs(*it))
                }

                // n children
                VecLang::List(ids)
                | VecLang::Vec(ids)
                // | VecLang::LitVec(ids)
		    => {
                    ids.iter().fold(0.0, |acc, it| acc + costs(*it))
                }
            };

        // add small amount to raw_cost in case it's 0
        1.0 / (raw_tot_cost + 0.01)
    }
}

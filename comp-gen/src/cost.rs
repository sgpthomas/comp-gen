use std::marker::PhantomData;

use egg;

use crate::FromPattern;

/// Metrics defined on an `egg::CostFunction` that are useful for categorizing rules.
pub trait CostMetrics<L, N>: egg::CostFunction<L>
where
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
{
    fn cost_differential(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
    fn cost_average(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
    fn all(&mut self, r: &egg::Rewrite<L, N>) -> CostMetric<L, N, Self>
    where
        Self: Sized,
    {
        CostMetric {
            cd: self.cost_differential(r),
            ca: self.cost_average(r),
            phantom: PhantomData,
        }
    }
}

/// Find the depth of a `RecExpr` starting at a particular `root` node.
#[allow(unused)]
pub fn depth<L: egg::Language, I: Into<egg::Id>>(
    term: &egg::RecExpr<L>,
    root: I,
) -> usize {
    if term.is_dag() {
        1 + term[root.into()]
            .fold(0, |acc, id| std::cmp::max(acc, depth(term, id)))
    } else {
        usize::max_value()
    }
}

pub struct CostMetric<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    pub cd: C::Cost,
    pub ca: C::Cost,
    phantom: PhantomData<N>,
}

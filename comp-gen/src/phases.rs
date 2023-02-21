use derivative::Derivative;
use ruler::egg;

use crate::{CostMetric, FromPattern};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SinglePhase<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    /// The name of the phase.
    pub(crate) name: String,
    // rules: Vec<egg::Rewrite<L, N>>,
    /// Predicate that selects which rules should run in this phase.
    #[derivative(Debug = "ignore")]
    pub(crate) select: Box<dyn Fn(CostMetric<L, N, C>) -> bool>,
    /// Should this phase use a new egraph when performing saturation?
    pub(crate) fresh_egraph: bool,
    /// The node limit for the egraph used in this phsae.
    pub(crate) node_limit: Option<usize>,
    /// The iter limit for the egraph used in this phase.
    pub(crate) iter_limit: Option<usize>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Phase<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    Single(SinglePhase<L, N, C>),
    Loop {
        phases: Vec<Phase<L, N, C>>,
        loops: usize,
    },
}

impl<L, N, C> Default for Phase<L, N, C>
where
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
{
    fn default() -> Self {
        Phase::Loop {
            phases: vec![],
            loops: 1,
        }
    }
}

// pub struct PhaseIter<L, N, C>;

// impl<L, N, C> Iterator for PhaseIter<L, N, C>
// where
//     L: egg::Language + FromPattern,
//     N: egg::Analysis<L>,
//     C: egg::CostFunction<L>,
// {
//     type Item = SinglePhase<L, N, C>;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

pub struct PhaseBuilder<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    phases: Vec<Phase<L, N, C>>,
}

impl<
        L: egg::Language + FromPattern,
        N: egg::Analysis<L>,
        C: egg::CostFunction<L>,
    > Default for PhaseBuilder<L, N, C>
{
    fn default() -> Self {
        PhaseBuilder { phases: vec![] }
    }
}

impl<
        L: egg::Language + FromPattern,
        N: egg::Analysis<L>,
        C: egg::CostFunction<L>,
    > PhaseBuilder<L, N, C>
{
    pub fn add_single<S, F>(&mut self, name: S, select: F) -> &mut Self
    where
        S: ToString,
        F: Fn(CostMetric<L, N, C>) -> bool + 'static,
    {
        let single_phase = SinglePhase {
            name: name.to_string(),
            select: Box::new(select),
            fresh_egraph: false,
            node_limit: None,
            iter_limit: None,
        };
        self.phases.push(Phase::Single(single_phase));
        self
    }

    pub fn add_single_w_opts<S, F>(
        &mut self,
        name: S,
        select: F,
        fresh_egraph: bool,
        node_limit: Option<usize>,
        iter_limit: Option<usize>,
    ) -> &mut Self
    where
        S: ToString,
        F: Fn(CostMetric<L, N, C>) -> bool + 'static,
    {
        let single_phase = SinglePhase {
            name: name.to_string(),
            select: Box::new(select),
            fresh_egraph,
            node_limit,
            iter_limit,
        };
        self.phases.push(Phase::Single(single_phase));
        self
    }

    pub fn add_loop(
        &mut self,
        phases: Vec<Phase<L, N, C>>,
        loops: usize,
    ) -> &mut Self {
        let loop_phase = Phase::Loop { phases, loops };
        self.phases.push(loop_phase);
        self
    }

    pub fn build_loop<F: FnOnce(&mut PhaseBuilder<L, N, C>)>(
        &mut self,
        loops: usize,
        f: F,
    ) -> &mut Self {
        let mut pb = PhaseBuilder::default();
        f(&mut pb);
        let loop_phase = Phase::Loop {
            phases: pb.phases,
            loops,
        };
        self.phases.push(loop_phase);
        self
    }

    pub fn build(mut self) -> Phase<L, N, C> {
        if self.phases.len() == 1 {
            self.phases.remove(0)
        } else {
            Phase::Loop {
                phases: self.phases,
                loops: 1,
            }
        }
    }
}

use derivative::Derivative;
use egg::*;

use crate::{config::RuleSchedulerOpt, CostMetric, FromPattern};

/// Describes a single phase of equality saturation. Besides a name, a phase
/// consists of the `select` predicate; which is a predicate selecting which rules
/// should be run in this phase. A phase can also specify an equality saturation
/// iteration and node limit. You can also specify that a fresh egraph should be
/// used for this phase. This means that the best program is extracted from the
/// current egraph, and then a new egraph is constructed from scratch.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SinglePhase<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    /// The name of the phase.
    pub(crate) name: String,
    /// Predicate that selects which rules should run in this phase.
    #[derivative(Debug = "ignore")]
    pub(crate) select: Box<dyn Fn(CostMetric<L, N, C>) -> bool>,
    /// Should this phase use a new egraph when performing saturation?
    pub(crate) fresh_egraph: bool,
    /// The node limit for the egraph used in this phsae.
    pub(crate) node_limit: Option<usize>,
    /// The iter limit for the egraph used in this phase.
    pub(crate) iter_limit: Option<usize>,
    /// The timeout for this phase.
    pub(crate) timeout: Option<usize>,
    /// The scheduler to use for this phase.
    pub(crate) scheduler: Option<RuleSchedulerOpt>,
}

/// Describes the phase config tree. A phase can either be a single phase, or a loop
/// of a sequence of phases.
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
        timeout: Option<usize>,
    },
}

/// The default phase is an empty phase that runs for a single iteration. This does
/// nothing.
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
            timeout: None,
        }
    }
}

/// The recommended way to programmatically construct phases.
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
    pub fn build_single<S, F>(&mut self, name: S, select: F) -> &mut Self
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
            timeout: None,
            scheduler: None,
        };
        self.phases.push(Phase::Single(single_phase));
        self
    }

    pub fn build_single_w_opts<S, F>(
        &mut self,
        name: S,
        select: F,
        fresh_egraph: bool,
        node_limit: Option<usize>,
        iter_limit: Option<usize>,
        timeout: Option<usize>,
        scheduler: Option<RuleSchedulerOpt>,
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
            timeout,
            scheduler,
        };
        self.phases.push(Phase::Single(single_phase));
        self
    }

    pub fn add_loop(
        &mut self,
        phases: Vec<Phase<L, N, C>>,
        loops: usize,
        timeout: Option<usize>,
    ) -> &mut Self {
        let loop_phase = Phase::Loop {
            phases,
            loops,
            timeout,
        };
        self.phases.push(loop_phase);
        self
    }

    pub fn build_loop<F: FnOnce(&mut PhaseBuilder<L, N, C>)>(
        &mut self,
        loops: usize,
        timeout: Option<usize>,
        f: F,
    ) -> &mut Self {
        let mut pb = PhaseBuilder::default();
        f(&mut pb);
        let loop_phase = Phase::Loop {
            phases: pb.phases,
            loops,
            timeout,
        };
        self.phases.push(loop_phase);
        self
    }

    pub fn finish(mut self) -> Phase<L, N, C> {
        if self.phases.len() == 1 {
            self.phases.remove(0)
        } else {
            Phase::Loop {
                phases: self.phases,
                loops: 1,
                timeout: None,
            }
        }
    }
}

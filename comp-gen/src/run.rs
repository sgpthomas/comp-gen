use log::{debug, info, warn};
use ruler::egg;

use crate::{
    compiler,
    config::RuleSchedulerOpt,
    cost,
    phases::{Phase, SinglePhase},
    stats::{EggStats, Stats},
    CostMetrics, FromPattern,
};

pub struct EqSatResult<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    cost: C::Cost,
    prog: egg::RecExpr<L>,
    egraph: egg::EGraph<L, N>,
    // stats: stats::Stats<L, C>,
}

// pub trait Saturate<L, N, C>
// where
//     L: egg::Language + FromPattern,
//     N: egg::Analysis<L>,
//     C: egg::CostFunction<L>,
// {
//     fn equality_saturate();
//     fn run_phase(
//         &self,
//         phase: SinglePhase<L, N, C>,
//         eqsat: EqSatResult<L, N, C>,
//     ) -> EqSatResult<L, N, C>;
//     fn compile();
// }

impl<L, N, C> compiler::Compiler<L, N, C>
where
    L: egg::Language
        + egg::FromOp
        + Send
        + Sync
        + FromPattern
        + std::fmt::Display
        + 'static,
    N: egg::Analysis<L> + Default + Clone + std::fmt::Debug,
    C: CostMetrics<L, N> + egg::CostFunction<L> + Clone + std::fmt::Debug,
    <C as egg::CostFunction<L>>::Cost: PartialOrd<f64>,
{
    fn equality_saturate(
        &self,
        phase: &SinglePhase<L, N, C>,
        eqsat: EqSatResult<L, N, C>,
    ) -> EqSatResult<L, N, C> {
        let msg = format!("Starting Phase {}", phase.name);
        info!("{}", "=".repeat(msg.len()));
        info!("{msg}");

        // gather rules that pass the filter
        let mut cost_fn = self.cost_fn.clone();
        let rules = self
            .rules
            .iter()
            .filter(|r| (phase.select)(cost_fn.all(r)))
            .cloned()
            .collect::<Vec<_>>();
        info!("Using {} rules", rules.len());
        info!("{}", "=".repeat(msg.len()));

        if self.dump_rules {
            for r in &rules {
                info!(
                    "[cd:{:.2?} ca:{:.2?}] {} => {} ({})",
                    cost_fn.cost_differential(r),
                    cost_fn.cost_average(r),
                    r.searcher.get_pattern_ast().unwrap(),
                    r.applier.get_pattern_ast().unwrap(),
                    r.name
                );
            }
        }

        if self.dry_run {
            warn!("Dry run! So skipping this phase.");
            return eqsat;
        }

        info!("Making runner");
        info!(
            "Initial Program Depth: {}",
            cost::depth(&eqsat.prog, eqsat.prog.as_ref().len() - 1)
        );

        let EqSatResult {
            cost: old_cost,
            prog,
            mut egraph,
        } = eqsat;

        // update egraph
        if phase.fresh_egraph || !self.reuse_egraphs {
            info!("Using a fresh egraph for this phase");
            egraph = self.new_egraph();
        }

        let mut runner: egg::Runner<L, N, ()> =
            egg::Runner::new(Default::default())
                .with_egraph(egraph)
                .with_expr(&prog)
                .with_node_limit(
                    phase.node_limit.unwrap_or(self.total_node_limit),
                )
                .with_iter_limit(
                    phase.iter_limit.unwrap_or(self.total_iter_limit),
                )
                .with_time_limit(std::time::Duration::from_secs(self.timeout));

        // set the scheduler according to the options
        runner = match self.scheduler {
            RuleSchedulerOpt::Backoff => {
                runner.with_scheduler(egg::BackoffScheduler::default())
            }
            RuleSchedulerOpt::Simple => {
                runner.with_scheduler(egg::SimpleScheduler)
            }
        };

        // Add EggStats hook if we have a stats path
        runner = if let Some(_) = self.stats {
            runner.with_hook(EggStats::density())
        } else {
            runner
        };

        debug!("Starting equality saturation");
        runner = runner.run(&rules);

        // extract the best program
        debug!("Extracting best program");
        let extractor =
            egg::Extractor::new(&runner.egraph, self.cost_fn.clone());
        let (cost, prog) = extractor.find_best(runner.roots[0]);

        debug!("Egraph size: {}", runner.egraph.total_size());
        debug!(
            "Final Program Depth: {}",
            cost::depth(&prog, prog.as_ref().len() - 1)
        );

        // Report some stats about this phase
        let stats = Stats::from_runner(
            &phase,
            rules.len(),
            &runner,
            old_cost,
            cost.clone(),
        );
        stats.report();

        EqSatResult {
            cost,
            prog,
            egraph: runner.egraph,
        }
    }

    fn run_phase(
        &self,
        phase: &Phase<L, N, C>,
        mut eqsat: EqSatResult<L, N, C>,
    ) -> EqSatResult<L, N, C> {
        match phase {
            crate::phases::Phase::Single(single) => {
                eqsat = self.equality_saturate(single, eqsat)
            }
            crate::phases::Phase::Loop { phases, loops } => {
                for i in 0..*loops {
                    info!("loop {i}");
                    for p in phases {
                        eqsat = self.run_phase(p, eqsat);
                    }
                }
            }
        }
        eqsat
    }

    pub fn compile(
        &mut self,
        prog: egg::RecExpr<L>,
    ) -> (C::Cost, egg::RecExpr<L>, egg::EGraph<L, N>) {
        // initialize eqsat to the default egraph and the
        // program that we were given
        let eqsat = EqSatResult {
            cost: self.cost_fn.cost_rec(&prog),
            prog,
            egraph: self.new_egraph(),
        };

        let eqsat = self.run_phase(&self.phases, eqsat);
        (eqsat.cost, eqsat.prog, eqsat.egraph)
    }
}

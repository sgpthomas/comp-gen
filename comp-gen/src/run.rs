use std::time::{Duration, Instant};

use egg;
use log::{debug, info, warn};

use crate::{
    compiler,
    config::RuleSchedulerOpt,
    phases::{Phase, SinglePhase},
    stats::{EggStats, Stats},
    CostMetrics, FromPattern,
};

pub struct EqSatResult<
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
    C: egg::CostFunction<L>,
> {
    pub cost: C::Cost,
    pub prog: egg::RecExpr<L>,
    pub egraph: egg::EGraph<L, N>,
    pub time_left: Duration,
    pub roots: Vec<egg::Id>,
}

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
    C: CostMetrics<L, N>
        + egg::CostFunction<L>
        + Clone
        + std::fmt::Debug
        + 'static,
    <C as egg::CostFunction<L>>::Cost: PartialOrd<f64>,
    <C as egg::CostFunction<L>>::Cost: PartialEq,
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
                debug!(
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

        // unpack the previous eqsat result
        let EqSatResult {
            cost: old_cost,
            prog: old_prog,
            mut egraph,
            time_left,
            roots: _old_roots,
        } = eqsat;

        // update egraph
        if phase.fresh_egraph || !self.reuse_egraphs {
            info!("Using a fresh egraph for this phase");
            egraph = self.new_egraph();
        }

        // choose a timeout. If we have a phase timeout,
        //   timeout = min(phase_timeout, time_left)
        // else
        //   timeout = time_left
        let timeout = if let Some(phase_timeout) = phase.timeout {
            time_left.min(Duration::from_secs(phase_timeout as u64))
        } else {
            time_left
        };

        info!("Making runner");
        let iter_cost_fn = self.cost_fn.clone();
        let mut runner: egg::Runner<L, N, ()> =
            egg::Runner::new(Default::default())
                .with_egraph(egraph)
                .with_expr(&old_prog)
                .with_node_limit(
                    phase.node_limit.unwrap_or(self.total_node_limit),
                )
                .with_iter_limit(
                    phase.iter_limit.unwrap_or(self.total_iter_limit),
                )
                .with_time_limit(timeout);
        debug!("Time left: {:?}", time_left);
        debug!("Using timeout: {:?}", timeout);

        runner = if self.debug {
            runner.with_hook(move |runner| {
                let start = Instant::now();
                let extractor =
                    egg::Extractor::new(&runner.egraph, iter_cost_fn.clone());
                let (cost, prog) = extractor.find_best(runner.roots[0]);
                let duration = start.elapsed();
                info!("Best cost so far: {cost:?}");
                info!("Best program: {prog}");
                info!("Extraction took: {duration:?}");
                Ok(())
            })
        } else {
            runner
        };

        // set the scheduler according to the options
        runner = match phase.scheduler.as_ref().unwrap_or(&self.scheduler) {
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
        let (cost, mut prog) = extractor.find_best(runner.roots[0]);
        // if the cost hasn't changed, just return the old program
        // this ensures that "unimportant changes" (i.e changes not captured by the cost function)
        // don't affect the cycle estimate results due to random reordering of expressions
        if old_cost == cost {
            prog = old_prog;
        }

        debug!("Egraph size: {}", runner.egraph.total_size());

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
            time_left: time_left
                .saturating_sub(Duration::from_secs_f64(stats.total_time)),
            roots: runner.roots,
        }
    }

    /// Recursively walk over the phase definitions, calling `self.equality_saturate`
    /// on the leaf phases. This is the function that threads through egraphs and progs
    /// through the different phases.
    fn run_phase(
        &self,
        phase: &Phase<L, N, C>,
        mut eqsat: EqSatResult<L, N, C>,
    ) -> EqSatResult<L, N, C> {
        match phase {
            Phase::Single(single) => {
                eqsat = self.equality_saturate(single, eqsat)
            }
            Phase::Loop {
                phases,
                loops,
                timeout,
            } => {
                let orig_time_left = eqsat.time_left.clone();
                if let Some(to) = timeout {
                    info!("Loop has timeout as {to}s; with {:?} total time remaining.", eqsat.time_left);
                    eqsat.time_left =
                        eqsat.time_left.min(Duration::from_secs(*to as u64));
                }
                'outer: for i in 0..*loops {
                    info!("loop {i}");
                    let old_cost = eqsat.cost.clone();
                    // if this loop has a timeout, set time_left to be the loop timeout
                    for p in phases {
                        eqsat = self.run_phase(p, eqsat);
                        if eqsat.time_left.is_zero() {
                            info!("Loop timed out, stopping early!");
                            break 'outer;
                        }
                    }
                    if old_cost == eqsat.cost {
                        info!("Cost didn't change from this iteration, stopping early!");
                        break;
                    }
                }

                if let Some(to) = timeout {
                    eqsat.time_left = orig_time_left
                        .saturating_sub(Duration::from_secs(*to as u64));
                    info!("Reseting timeout to {:?}", eqsat.time_left);
                }
            }
        }
        eqsat
    }

    pub fn compile(&mut self, prog: egg::RecExpr<L>) -> EqSatResult<L, N, C> {
        log::debug!("Phase config: {:#?}", self.phases);
        self.generate_rule_histogram();

        // initialize eqsat to the default egraph and the
        // program that we were given
        let eqsat = EqSatResult {
            cost: self.cost_fn.cost_rec(&prog),
            prog,
            egraph: self.new_egraph(),
            time_left: Duration::from_secs(self.timeout),
            roots: vec![],
        };

        // (eqsat.cost, eqsat.prog, eqsat.egraph)
        self.run_phase(&self.phases, eqsat)
    }
}

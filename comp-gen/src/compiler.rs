use ruler::egg::rewrite as rw;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use ruler::egg;

use crate::{
    config,
    cost::CostMetrics,
    phases::{self, Phase},
    FromPattern,
};

/// An equality saturation based compiler.
pub struct Compiler<
    L: egg::Language + egg::FromOp + Send + Sync + FromPattern + 'static,
    N: egg::Analysis<L> + Default + Clone,
    C: egg::CostFunction<L> + Clone,
> {
    /// All of the rewrite rules for this compiler.
    pub(crate) rules: Vec<egg::Rewrite<L, N>>,

    /// Optional function that filters valid rewrite rules.
    // filter: Option<Box<dyn Fn(CostMetric<L, N, C>) -> bool>>,
    pub(crate) phases: phases::Phase<L, N, C>,
    pub(crate) cost_fn: C,
    pub(crate) init_node: Option<L>,
    pub(crate) total_node_limit: usize,
    pub(crate) total_iter_limit: usize,
    pub(crate) timeout: u64,
    pub(crate) dry_run: bool,
    pub(crate) dump_rules: bool,
    pub(crate) reuse_egraphs: bool,
    pub(crate) rule_distribution: Vec<(PathBuf, Box<dyn Fn(C::Cost) -> f64>)>,
    pub(crate) explanations: bool,
    pub(crate) scheduler: config::RuleSchedulerOpt,
    pub(crate) stats: Option<PathBuf>,
}

impl<L, N, C> Compiler<L, N, C>
where
    L: egg::Language
        + egg::FromOp
        + Send
        + Sync
        + FromPattern
        + std::fmt::Display
        + 'static,
    N: egg::Analysis<L> + Default + Clone,
    C: CostMetrics<L, N> + egg::CostFunction<L> + Clone,
    <C as egg::CostFunction<L>>::Cost: PartialOrd<f64>,
{
    /// Construct a compiler using `cost_fn` to distinguish
    /// between the `source` and `target` languages.
    /// Defaults to using a `node_limit=1_000_000` and
    /// a `timeout=120 seconds`.
    pub fn with_cost_fn(cost_fn: C) -> Self {
        Self {
            rules: vec![],
            phases: Phase::default(),
            // filter: None,
            cost_fn,
            init_node: None,
            total_node_limit: 1_000_000,
            total_iter_limit: 30,
            timeout: 120,
            dry_run: false,
            dump_rules: false,
            reuse_egraphs: true,
            rule_distribution: vec![],
            explanations: false,
            scheduler: config::RuleSchedulerOpt::default(),
            stats: None,
        }
    }

    /// Return rules read in from a json file.
    pub fn add_external_rules(&mut self, filename: &Path) -> &mut Self {
        self.add_processed_external_rules(filename, |x| x)
    }

    pub fn add_processed_external_rules<F>(
        &mut self,
        filename: &Path,
        proc: F,
    ) -> &mut Self
    where
        F: Fn(egg::Pattern<L>) -> egg::Pattern<L>,
    {
        let contents = std::fs::read_to_string(filename).unwrap();
        let data = json::parse(&contents).unwrap();

        let mut rules = vec![];
        for (idx, eq) in data["eqs"].members().enumerate() {
            let lpat_raw: egg::Pattern<L> =
                eq["lhs"].as_str().unwrap().parse().unwrap();
            let rpat_raw: egg::Pattern<L> =
                eq["rhs"].as_str().unwrap().parse().unwrap();

            let lpat = proc(lpat_raw);
            let rpat = proc(rpat_raw);

            if eq["bidirectional"].as_bool().unwrap() {
                // we have to clone bc it is a bidirectional rule
                rules
                    .extend(rw!(format!("ruler_{}_lr", idx); { lpat.clone() } <=> { rpat.clone() }))
            } else {
                rules
                    .push(rw!(format!("ruler_{}_r", idx); { lpat } => { rpat }))
            }
        }

        self.rules.extend(rules);
        self
    }

    pub fn add_rules(
        &mut self,
        rules: impl Iterator<Item = egg::Rewrite<L, N>>,
    ) -> &mut Self {
        self.rules.extend(rules);
        self
    }

    pub fn with_init_node(&mut self, node: L) -> &mut Self {
        self.init_node = Some(node);
        self
    }

    pub fn with_total_node_limit(&mut self, node_limit: usize) -> &mut Self {
        self.total_node_limit = node_limit;
        self
    }

    pub fn with_total_iter_limit(&mut self, iter_limit: usize) -> &mut Self {
        self.total_iter_limit = iter_limit;
        self
    }

    pub fn with_timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }

    // pub fn with_filter<F: 'static>(&mut self, filter: F) -> &mut Self
    // where
    //     F: Fn(CostMetric<L, N, C>) -> bool,
    // {
    //     self.filter = Some(Box::new(filter));
    //     self
    // }

    pub fn with_phase_builder<F>(&mut self, build: F) -> &mut Self
    where
        F: FnOnce(&mut phases::PhaseBuilder<L, N, C>),
    {
        let mut pb = phases::PhaseBuilder::default();
        build(&mut pb);
        self.phases = pb.build();
        self
    }

    pub fn dry_run(&mut self) -> &mut Self {
        self.dry_run = true;
        self
    }

    pub fn dump_rules(&mut self) -> &mut Self {
        self.dump_rules = true;
        self
    }

    pub fn output_rule_distribution<P, F>(
        &mut self,
        output_file: P,
        f: F,
    ) -> &mut Self
    where
        P: Into<PathBuf>,
        F: Fn(C::Cost) -> f64 + 'static,
    {
        self.rule_distribution
            .push((output_file.into(), Box::new(f)));
        self
    }

    pub fn reuse_egraphs(&mut self) -> &mut Self {
        self.reuse_egraphs = true;
        self
    }

    /// Load config from a file. This overries anything previously set.
    pub fn with_config(
        &mut self,
        config: &config::CompilerConfiguration,
    ) -> &mut Self {
        self.total_node_limit = config.total_node_limit;
        self.total_iter_limit = config.total_iter_limit;
        self.timeout = config.timeout;
        self.dry_run = config.dry_run;
        self.dump_rules = config.dump_rules;
        self.reuse_egraphs = config.reuse_egraphs;
        self.phases = config.phase.clone().into();
        self.scheduler = config
            .scheduler
            .clone()
            .unwrap_or(config::RuleSchedulerOpt::default());
        self.stats = config.stats.clone();
        self
    }

    pub fn with_explanations(&mut self) -> &mut Self {
        self.explanations = true;
        self
    }

    pub fn with_scheduler(
        &mut self,
        scheduler: config::RuleSchedulerOpt,
    ) -> &mut Self {
        self.scheduler = scheduler;
        self
    }

    pub fn with_stats_path(&mut self, path: PathBuf) -> &mut Self {
        self.stats = Some(path);
        self
    }

    // fn equality_saturate(
    //     &self,
    //     phase: &SinglePhase<L, N, C>,
    //     egraph: egg::EGraph<L, N>,
    //     prog: &egg::RecExpr<L>,
    // ) -> (egg::EGraph<L, N>, EqSatResult<L, C>) {
    //     debug!("Making runner");
    //     debug!(
    //         "Initial Program Depth: {}",
    //         cost::depth(&prog, prog.as_ref().len() - 1)
    //     );

    //     let mut runner: egg::Runner<L, N, ()> =
    //         egg::Runner::new(Default::default())
    //             .with_egraph(egraph)
    //             .with_expr(prog)
    //             .with_node_limit(phase.node_limit)
    //             .with_iter_limit(phase.iter_limit)
    //             .with_time_limit(std::time::Duration::from_secs(self.timeout));

    //     // set the scheduler according to the options
    //     runner = match self.scheduler {
    //         config::RuleSchedulerOpt::Backoff => {
    //             runner.with_scheduler(egg::BackoffScheduler::default())
    //         }
    //         config::RuleSchedulerOpt::Simple => {
    //             runner.with_scheduler(egg::SimpleScheduler)
    //         }
    //     };

    //     // Add EggStats hook if we have a stats path
    //     runner = if let Some(_) = self.stats {
    //         runner.with_hook(EggStats::density())
    //     } else {
    //         runner
    //     };

    //     let rules = self
    //         .rules
    //         .iter()
    //         .filter(|r| (phase.select)(self.cost_fn.clone().all(r)))
    //         .cloned()
    //         .collect::<Vec<_>>();

    //     debug!("Starting equality saturation");
    //     runner = runner.run(&rules);

    //     // extract the best program
    //     let extractor =
    //         egg::Extractor::new(&runner.egraph, self.cost_fn.clone());
    //     let (cost, prog) = extractor.find_best(runner.roots[0]);

    //     debug!("Egraph size: {}", runner.egraph.total_size());
    //     debug!(
    //         "Final Program Depth: {}",
    //         depth(&prog, prog.as_ref().len() - 1)
    //     );
    //     let mut stats = Stats::from_runner(&phase, rules.len(), &runner);
    //     stats.cost = Some(cost.clone());
    //     stats.report();

    //     let res = EqSatResult {
    //         cost,
    //         prog,
    //         // egraph: runner.egraph,
    //         stats,
    //     };
    //     (runner.egraph, res)
    // }

    // fn run_single(
    //     &self,
    //     phase: &SinglePhase<L, N, C>,
    //     mut egraph: egg::EGraph<L, N>,
    //     prog: &egg::RecExpr<L>,
    // ) -> (egg::EGraph<L, N>, Option<EqSatResult<L, C>>) {
    //     let msg = format!("Starting Phase {}", phase.name);
    //     info!("{}", "=".repeat(msg.len()));
    //     info!("{msg}");
    //     // gather rules that pass the filter
    //     let mut cost_fn = self.cost_fn.clone();
    //     let rules = self
    //         .rules
    //         .iter()
    //         .filter(|r| (phase.select)(cost_fn.all(r)))
    //         .cloned()
    //         .collect::<Vec<_>>();
    //     info!("Using {} rules", rules.len());
    //     info!("{}", "=".repeat(msg.len()));

    //     if self.dump_rules {
    //         for r in &rules {
    //             debug!(
    //                 "[cd:{:.2?} ca:{:.2?}] {} => {} ({})",
    //                 cost_fn.cost_differential(r),
    //                 cost_fn.cost_average(r),
    //                 r.searcher.get_pattern_ast().unwrap(),
    //                 r.applier.get_pattern_ast().unwrap(),
    //                 r.name
    //             );
    //         }
    //     }

    //     if self.dry_run {
    //         return (egraph, None);
    //     }

    //     // update egraph
    //     if phase.fresh_egraph || !self.reuse_egraphs {
    //         debug!("Using a fresh egraph for this phase");
    //         egraph = self.new_egraph();
    //     }
    //     // let (new_cost, new_prog, new_egraph) =
    //     //     self.equality_saturate(&phase, egraph, &prog);
    //     let (egraph, eq_sat) = self.equality_saturate(&phase, egraph, &prog);

    //     (egraph, Some(eq_sat))
    // }

    // fn run_loop(
    //     &self,
    //     phases: &[SinglePhase<L, N, C>],
    //     loops: u64,
    //     mut egraph: egg::EGraph<L, N>,
    //     prog: &egg::RecExpr<L>,
    // ) -> (egg::EGraph<L, N>, Option<EqSatResult<L, C>>) {
    //     info!("Starting a loop! Running for {loops} iterations");

    //     let mut res = None;

    //     for _ in 0..loops {
    //         for phase in phases {
    //             let (egraph_new, eq_sat_opt) =
    //                 self.run_single(phase, egraph, prog);
    //             egraph = egraph_new;
    //             res = eq_sat_opt;
    //             // handle stats if we have them
    //             // if let Some(eq_sat) = eq_sat_opt {
    //             //     let mut stat = eq_sat.stats.clone();
    //             //     stat.old_cost = Some(cost.clone());
    //             //     stats.push(stat);

    //             //     info!("Cost: {:?} (old: {:?})", eq_sat.cost.clone(), cost);
    //             //     // update state for next iteration
    //             //     // (cost, prog) = (eq_sat.cost, eq_sat.prog);
    //             // }
    //         }
    //     }
    //     (egraph, res)
    // }

    // pub fn compile(
    //     &mut self,
    //     prog: &egg::RecExpr<L>,
    // ) -> (C::Cost, egg::RecExpr<L>, egg::EGraph<L, N>) {
    //     self.rule_distribution.iter().for_each(|(path, conv)| {
    //         self.generate_rule_histogram(path, conv);
    //     });

    //     let mut egraph = self.new_egraph();
    //     // let mut cost_fn = self.cost_fn.clone();
    //     let mut prog = prog.clone();
    //     let mut cost = self.cost_fn.cost_rec(&prog);

    //     let mut stats: Vec<Stats<L, C>> = vec![];
    //     for phase in &self.phases {
    //         let (egraph_new, eq_sat_opt) = match phase {
    //             Phase::Single(phase) => self.run_single(phase, egraph, &prog),
    //             Phase::Loop { phases, loops } => {
    //                 self.run_loop(phases, *loops, egraph, &prog)
    //             }
    //         };
    //         // update the egraph with the one returned from run_single
    //         egraph = egraph_new;
    //         // handle stats if we have them
    //         if let Some(eq_sat) = eq_sat_opt {
    //             let mut stat = eq_sat.stats.clone();
    //             stat.old_cost = Some(cost.clone());
    //             stats.push(stat);

    //             info!("Cost: {:?} (old: {:?})", eq_sat.cost.clone(), cost);
    //             // update state for next iteration
    //             (cost, prog) = (eq_sat.cost, eq_sat.prog);
    //         }
    //     }

    //     // report the total stats
    //     for s in stats {
    //         s.report()
    //     }

    //     (cost, prog, egraph)
    // }

    pub(crate) fn new_egraph(&self) -> egg::EGraph<L, N> {
        let mut egraph = if self.explanations {
            egg::EGraph::new(N::default()).with_explanations_enabled()
        } else {
            egg::EGraph::new(N::default()).with_explanations_disabled()
        };
        if let Some(node) = &self.init_node {
            egraph.add(node.clone());
        }
        egraph
    }

    #[allow(unused)]
    fn generate_rule_histogram<F>(&self, path: &PathBuf, f: F)
    where
        F: Fn(C::Cost) -> f64,
    {
        let mut cost_fn = self.cost_fn.clone();
        let avg_data: Vec<(String, f64)> = self
            .rules
            .iter()
            .map(|r| (r.name.to_string(), f(cost_fn.cost_average(r))))
            .collect();
        let diff_data: Vec<(String, f64)> = self
            .rules
            .iter()
            .map(|r| (r.name.to_string(), f(cost_fn.cost_differential(r))))
            .collect();

        let mut file = File::create(path).unwrap();

        writeln!(file, "rule,name,value").unwrap();

        for (r, d) in avg_data {
            writeln!(file, "{r},average,{d}").unwrap();
        }

        for (r, d) in diff_data {
            writeln!(file, "{r},differential,{d}").unwrap();
        }
    }
}

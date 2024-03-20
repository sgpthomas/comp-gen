use egg::{self, rewrite as rw};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

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
    pub(crate) phases: phases::Phase<L, N, C>,
    pub(crate) cost_fn: C,
    pub(crate) init_node: Option<L>,
    pub(crate) total_node_limit: usize,
    pub(crate) total_iter_limit: usize,
    pub(crate) timeout: u64,
    pub(crate) dry_run: bool,
    pub(crate) dump_rules: bool,
    pub(crate) debug: bool,
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
            cost_fn,
            init_node: None,
            total_node_limit: 1_000_000,
            total_iter_limit: 30,
            timeout: 120,
            dry_run: false,
            debug: false,
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

    pub fn with_phase_builder<F>(&mut self, build: F) -> &mut Self
    where
        F: FnOnce(&mut phases::PhaseBuilder<L, N, C>),
    {
        let mut pb = phases::PhaseBuilder::default();
        build(&mut pb);
        self.phases = pb.finish();
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

    pub fn display_rules(&self) {
        for r in &self.rules {
            log::info!(
                "{} => {} ({})",
                r.searcher.get_pattern_ast().unwrap(),
                r.applier.get_pattern_ast().unwrap(),
                r.name
            );
        }
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
        self.debug = config.debug;
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

    pub(crate) fn generate_rule_histogram(&self) {
        for (path, f) in self.rule_distribution.iter() {
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
}

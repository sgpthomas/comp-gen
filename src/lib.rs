use egg::rewrite as rw;
use std::io::Write;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub trait Interpreter {
    type Env: Default;
    type Res;

    fn eval_with_env(&self, env: &mut Self::Env) -> Self::Res;
    fn eval(&self) -> Self::Res {
        self.eval_with_env(&mut Self::Env::default())
    }
}

pub trait ToRecExpr<T> {
    fn to_recexpr(&self, expr: &mut egg::RecExpr<T>) -> egg::Id;
}

pub trait FromPattern: Sized {
    fn from_pattern(pat: &egg::PatternAst<Self>) -> egg::RecExpr<Self>;
}

pub trait CostMetrics<L, N>: egg::CostFunction<L>
where
    L: egg::Language + FromPattern,
    N: egg::Analysis<L>,
{
    fn cost_differential(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
    fn cost_average(&mut self, r: &egg::Rewrite<L, N>) -> Self::Cost;
}

pub struct Compiler<
    L: egg::Language + egg::FromOp + Send + Sync + FromPattern + 'static,
    N: egg::Analysis<L> + Default + Clone,
    C: egg::CostFunction<L> + Clone,
> {
    rules: Vec<egg::Rewrite<L, N>>,
    phases: Vec<(String, Box<dyn Fn(C::Cost, C::Cost) -> bool>)>,
    cost_fn: C,
    init_node: Option<L>,
    node_limit: usize,
    timeout: u64,
    dry_run: bool,
    dump_rules: bool,
    rule_distribution: Vec<(PathBuf, Box<dyn Fn(C::Cost) -> f64>)>,
    reuse_egraphs: bool,
}

impl<
        L: egg::Language + egg::FromOp + Send + Sync + FromPattern + std::fmt::Display + 'static,
        N: egg::Analysis<L> + Default + Clone,
        C: CostMetrics<L, N> + egg::CostFunction<L> + Clone,
    > Compiler<L, N, C>
{
    /// Construct a compiler using `cost_fn` to distinguish
    /// between the `source` and `target` languages.
    /// Defaults to using a `node_limit=1_000_000` and
    /// a `timeout=120 seconds`.
    pub fn with_cost_fn(cost_fn: C) -> Self {
        Self {
            rules: vec![],
            phases: vec![],
            cost_fn,
            init_node: None,
            node_limit: 1_000_000,
            timeout: 120,
            dry_run: false,
            dump_rules: false,
            rule_distribution: vec![],
            reuse_egraphs: false,
        }
    }

    /// Return rules read in from a json file.
    pub fn add_external_rules(mut self, filename: &Path) -> Self {
        let contents = std::fs::read_to_string(filename).unwrap();
        let data = json::parse(&contents).unwrap();

        let mut rules = vec![];
        for (idx, eq) in data["eqs"].members().enumerate() {
            let lpat: egg::Pattern<L> = eq["lhs"].as_str().unwrap().parse().unwrap();
            let rpat: egg::Pattern<L> = eq["rhs"].as_str().unwrap().parse().unwrap();

            if eq["bidirectional"].as_bool().unwrap() {
                // we have to clone bc it is a bidirectional rule
                rules
                    .extend(rw!(format!("ruler_{}_lr", idx); { lpat.clone() } <=> { rpat.clone() }))
            } else {
                rules.push(rw!(format!("ruler_{}_r", idx); { lpat } => { rpat }))
            }
        }

        self.rules = rules;
        self
    }

    pub fn add_rules(mut self, rules: impl Iterator<Item = egg::Rewrite<L, N>>) -> Self {
        self.rules.extend(rules);
        self
    }

    pub fn with_init_node(mut self, node: L) -> Self {
        self.init_node = Some(node);
        self
    }

    pub fn with_node_limit(mut self, node_limit: usize) -> Self {
        self.node_limit = node_limit;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn add_cutoff_phase<F: 'static>(mut self, name: &str, filter: F) -> Self
    where
        F: Fn(C::Cost, C::Cost) -> bool,
    {
        self.phases.push((name.to_string(), Box::new(filter)));
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn dump_rules(mut self) -> Self {
        self.dump_rules = true;
        self
    }

    pub fn output_rule_distribution<P, F>(mut self, output_file: P, f: F) -> Self
    where
        P: Into<PathBuf>,
        F: Fn(C::Cost) -> f64 + 'static,
    {
        self.rule_distribution
            .push((output_file.into(), Box::new(f)));
        self
    }

    pub fn reuse_egraphs(mut self) -> Self {
        self.reuse_egraphs = true;
        self
    }

    fn split_rules(&self) -> Vec<(&str, Vec<egg::Rewrite<L, N>>)> {
        if self.phases.is_empty() {
            vec![("all rules", self.rules.clone())]
        } else {
            let mut cost_fn = self.cost_fn.clone();
            self.phases
                .iter()
                .map(|(name, filter_fn)| {
                    let rules = self
                        .rules
                        .iter()
                        .filter(|r| {
                            let cost = cost_fn.cost_differential(r);
                            let avg = cost_fn.cost_average(r);
                            filter_fn(cost, avg)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    (name.as_str(), rules)
                })
                .collect::<Vec<_>>()
        }
    }

    fn equality_saturate(
        &self,
        rules: &[egg::Rewrite<L, N>],
        egraph: egg::EGraph<L, N>,
        prog: &egg::RecExpr<L>,
    ) -> (C::Cost, egg::RecExpr<L>, egg::EGraph<L, N>) {
        eprint!("Making runner...");
        let mut runner: egg::Runner<L, N, ()> = egg::Runner::new(Default::default())
            .with_egraph(egraph)
            .with_expr(prog)
            .with_node_limit(self.node_limit)
            .with_time_limit(std::time::Duration::from_secs(self.timeout));
        eprintln!("done");

        // TODO make scheduler an option
        // runner = runner.with_scheduler(egg::BackoffScheduler::default());
        runner = runner.with_scheduler(egg::SimpleScheduler);

        // let mut cost_fn = self.cost_fn.clone();
        // for r in rules {
        //     eprintln!(
        //         "[{:?} {:?}] {} <=> {} ({})",
        //         cost_fn.cost_differential(r),
        //         cost_fn.cost_average(r),
        //         r.searcher.get_pattern_ast().unwrap(),
        //         r.applier.get_pattern_ast().unwrap(),
        //         r.name
        //     );
        // }

        runner = runner.run(rules);

        eprintln!("Egraph size: {}", runner.egraph.total_size());
        self.report(&runner);

        // extract the best program
        let extractor = egg::Extractor::new(&runner.egraph, self.cost_fn.clone());
        let (cost, prog) = extractor.find_best(runner.roots[0]);
        (cost, prog, runner.egraph)
    }

    pub fn compile(
        &self,
        prog: &egg::RecExpr<L>,
    ) -> (Option<C::Cost>, egg::RecExpr<L>, egg::EGraph<L, N>) {
        self.rule_distribution.iter().for_each(|(path, conv)| {
            self.generate_rule_histogram(path, conv);
        });

        let mut egraph = self.new_egraph();
        let mut cost_fn = self.cost_fn.clone();
        let mut prog = prog.clone();
        let mut cost = None;
        for (phase_name, rules) in self.split_rules() {
            let msg = format!("Starting Phase {}", &phase_name);
            eprintln!("{}", "=".repeat(msg.len()));
            eprintln!("{msg}");
            eprintln!("Using {} rules", rules.len());
            eprintln!("{}", "=".repeat(msg.len()));

            if self.dump_rules {
                for r in &rules {
                    eprintln!(
                        "[cd:{:?} ca:{:?}] {} => {} ({})",
                        cost_fn.cost_differential(r),
                        cost_fn.cost_average(r),
                        r.searcher.get_pattern_ast().unwrap(),
                        r.applier.get_pattern_ast().unwrap(),
                        r.name
                    );
                }
            }

            if self.dry_run {
                continue;
            }

            let (new_cost, new_prog, new_egraph) = self.equality_saturate(&rules, egraph, &prog);

            if let Some(old_cost) = cost {
                eprintln!(
                    "Cost: {:?} (old: {:?})",
                    new_cost.clone(),
                    // (self.subtract)(old_cost, new_cost.clone())
                    old_cost
                );
            } else {
                eprintln!("Cost: {:?}", new_cost);
            }

            // update state for next iteration
            (cost, prog) = (Some(new_cost), new_prog);

            // update egraph
            if self.reuse_egraphs {
                egraph = new_egraph;
            } else {
                egraph = self.new_egraph();
            }
        }

        (cost, prog, egraph)
    }

    fn new_egraph(&self) -> egg::EGraph<L, N> {
        let mut egraph = egg::EGraph::new(N::default()).with_explanations_disabled();
        if let Some(node) = &self.init_node {
            egraph.add(node.clone());
        }
        egraph
    }

    fn report(&self, runner: &egg::Runner<L, N, ()>) {
        let search_time: f64 = runner.iterations.iter().map(|i| i.search_time).sum();
        let apply_time: f64 = runner.iterations.iter().map(|i| i.apply_time).sum();
        let rebuild_time: f64 = runner.iterations.iter().map(|i| i.rebuild_time).sum();
        let total_time: f64 = runner.iterations.iter().map(|i| i.total_time).sum();

        let iters = runner.iterations.len();
        let rebuilds: usize = runner.iterations.iter().map(|i| i.n_rebuilds).sum();

        let eg = &runner.egraph;
        eprintln!("  Runner report");
        eprintln!("  =============");
        eprintln!(
            "    Stop reason: {:?}",
            runner.stop_reason.as_ref().unwrap()
        );
        eprintln!("    Iterations: {}", iters);
        eprintln!(
            "    Egraph size: {} nodes, {} classes, {} memo",
            eg.total_number_of_nodes(),
            eg.number_of_classes(),
            eg.total_size()
        );
        eprintln!(
            "    Rebuilds: {}, {:.2} per iter",
            rebuilds,
            (rebuilds as f64) / (iters as f64)
        );
        eprintln!("    Total time: {}", total_time);
        eprintln!(
            "      Search:  ({:.2}) {}",
            search_time / total_time,
            search_time
        );
        eprintln!(
            "      Apply:   ({:.2}) {}",
            apply_time / total_time,
            apply_time
        );
        eprintln!(
            "      Rebuild: ({:.2}) {}",
            rebuild_time / total_time,
            rebuild_time
        );
    }

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

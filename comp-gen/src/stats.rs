use log::info;
use ruler::egg;

use crate::{phases::SinglePhase, FromPattern};
// use statrs::statistics::Statistics;

#[derive(Default)]
pub struct EggStats;

impl EggStats {
    pub fn density<L: egg::Language, N: egg::Analysis<L>>(
    ) -> impl FnMut(&mut egg::Runner<L, N>) -> Result<(), String> + 'static
    {
        |runner: &mut egg::Runner<L, N>| {
            let s1 = plotlib::repr::Histogram::from_slice(
                &runner
                    .egraph
                    .classes()
                    .map(|cls| cls.nodes.len() as f64)
                    .collect::<Vec<_>>(),
                plotlib::repr::HistogramBins::Count(30),
            );

            let v = plotlib::view::ContinuousView::new()
                .add(s1)
                .x_label("Count")
                .y_label("Bins");

            log::debug!(
                "\n{}",
                plotlib::page::Page::single(&v)
                    .dimensions(80, 80)
                    .to_text()
                    .unwrap()
            );
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stats<L: egg::Language, C: egg::CostFunction<L>> {
    phase_name: String,
    rules: usize,
    stop_reason: Option<egg::StopReason>,
    iterations: usize,
    egraph_total_nodes: usize,
    egraph_total_classes: usize,
    egraph_total_size: usize,
    old_cost: C::Cost,
    cost: C::Cost,
}

impl<L, C> Stats<L, C>
where
    L: egg::Language + egg::FromOp + Send + Sync + FromPattern + 'static,
    C: egg::CostFunction<L> + Clone,
{
    pub fn from_runner<N: egg::Analysis<L> + Default + Clone>(
        phase: &SinglePhase<L, N, C>,
        n_rules: usize,
        runner: &egg::Runner<L, N, ()>,
        old_cost: C::Cost,
        cost: C::Cost,
    ) -> Self {
        Self {
            phase_name: phase.name.to_string(),
            rules: n_rules,
            stop_reason: runner.stop_reason.clone(),
            iterations: runner.iterations.len(),
            egraph_total_nodes: runner.egraph.total_number_of_nodes(),
            egraph_total_classes: runner.egraph.number_of_classes(),
            egraph_total_size: runner.egraph.total_size(),
            old_cost,
            cost,
        }
    }

    pub fn report(&self) {
        // let search_time: f64 =
        //     runner.iterations.iter().map(|i| i.search_time).sum();
        // let apply_time: f64 =
        //     runner.iterations.iter().map(|i| i.apply_time).sum();
        // let rebuild_time: f64 =
        //     runner.iterations.iter().map(|i| i.rebuild_time).sum();
        // let total_time: f64 =
        //     runner.iterations.iter().map(|i| i.total_time).sum();

        // let rebuilds: usize =
        //     runner.iterations.iter().map(|i| i.n_rebuilds).sum();

        // let eg = &runner.egraph;
        info!("  Runner report");
        info!("  =============");
        info!("    Phase: '{}' with {} rules", self.phase_name, self.rules);
        info!("    Stop reason: {:?}", self.stop_reason.as_ref().unwrap());
        info!("    Iterations: {}", self.iterations);
        info!("    Cost: {:?} (old: {:?})", self.cost, self.old_cost);
        info!(
            "    Egraph size: {} nodes, {} classes, {} memo",
            self.egraph_total_nodes,
            self.egraph_total_classes,
            self.egraph_total_size
        );
        // eprintln!(
        //     "    Rebuilds: {}, {:.2} per iter",
        //     rebuilds,
        //     (rebuilds as f64) / (iters as f64)
        // );
        // eprintln!("    Total time: {}", total_time);
        // eprintln!(
        //     "      Search:  ({:.2}) {}",
        //     search_time / total_time,
        //     search_time
        // );
        // eprintln!(
        //     "      Apply:   ({:.2}) {}",
        //     apply_time / total_time,
        //     apply_time
        // );
        // eprintln!(
        //     "      Rebuild: ({:.2}) {}",
        //     rebuild_time / total_time,
        //     rebuild_time
        // );
    }
}

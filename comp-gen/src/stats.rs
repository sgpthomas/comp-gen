use ruler::egg;
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

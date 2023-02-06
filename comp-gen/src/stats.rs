use ruler::egg;
use statrs::statistics::Statistics;

#[derive(Default)]
pub struct EggStats;

impl EggStats {
    pub fn density<L: egg::Language, N: egg::Analysis<L>>(
    ) -> impl FnMut(&mut egg::Runner<L, N>) -> Result<(), String> + 'static
    {
        |runner: &mut egg::Runner<L, N>| {
            let s = runner
                .egraph
                .classes()
                .map(|cls| cls.nodes.len() as f64)
                .std_dev();
            let m = runner
                .egraph
                .classes()
                .map(|cls| cls.nodes.len() as f64)
                .mean();
            log::debug!("Mean: {m}, Stddev: {s}");
            Ok(())
        }
    }
}

use std::path::PathBuf;

use egg;
use serde::{Deserialize, Serialize};

use crate::{phases, FromPattern};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSchedulerOpt {
    Backoff,
    #[default]
    Simple,
}

/// A serializable configuration struct so that configurations can be loaded
/// from files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompilerConfiguration {
    pub total_node_limit: usize,
    pub total_iter_limit: usize,
    pub timeout: u64,
    pub dry_run: bool,
    pub debug: bool,
    pub dump_rules: bool,
    pub reuse_egraphs: bool,
    pub cd_filter: Option<f64>,
    pub require_all_vars: bool,
    pub phase: PhaseConfiguration,
    pub scheduler: Option<RuleSchedulerOpt>,
    pub stats: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PhaseConfiguration {
    Single {
        name: String,
        cd: [Option<f64>; 2],
        ca: [Option<f64>; 2],
        fresh_egraph: Option<bool>,
        node_limit: Option<usize>,
        iter_limit: Option<usize>,
        timeout: Option<usize>,
        disabled: Option<bool>,
        scheduler: Option<RuleSchedulerOpt>,
    },
    Phases {
        phases: Vec<PhaseConfiguration>,
        loops: Option<usize>,
        timeout: Option<usize>,
    },
}

impl<
        L: egg::Language + FromPattern,
        N: egg::Analysis<L>,
        C: egg::CostFunction<L>,
    > Into<phases::Phase<L, N, C>> for PhaseConfiguration
where
    <C as egg::CostFunction<L>>::Cost: PartialOrd<f64>,
{
    fn into(self) -> phases::Phase<L, N, C> {
        let mut pb = phases::PhaseBuilder::<L, N, C>::default();
        match self {
            PhaseConfiguration::Single {
                name,
                cd,
                ca,
                fresh_egraph,
                node_limit,
                iter_limit,
                timeout,
                disabled,
                scheduler,
            } => {
                if !disabled.unwrap_or(false) {
                    let [cd_low, cd_high] = cd;
                    let [ca_low, ca_high] = ca;
                    pb.build_single_w_opts(
                        name,
                        // check all the conditions, if a condition doesn't exist
                        // default to true for that check
                        move |cm| {
                            cd_low.map(|l| cm.cd > l).unwrap_or(true)
                                && cd_high.map(|h| cm.cd <= h).unwrap_or(true)
                                && ca_low.map(|l| cm.ca > l).unwrap_or(true)
                                && ca_high.map(|h| cm.ca <= h).unwrap_or(true)
                        },
                        fresh_egraph.unwrap_or(false),
                        node_limit,
                        iter_limit,
                        timeout,
                        scheduler,
                    );
                }
            }
            PhaseConfiguration::Phases {
                phases,
                loops,
                timeout,
            } => {
                pb.add_loop(
                    phases
                        .into_iter()
                        .map(|x| x.into())
                        // only keep non-empty phases
                        .filter(|p| match p {
                            phases::Phase::Single(_) => true,
                            phases::Phase::Loop { phases, .. } => {
                                phases.len() > 0
                            }
                        })
                        .collect::<Vec<_>>(),
                    loops.unwrap_or(1),
                    timeout,
                );
            }
        }
        pb.finish()
    }
}

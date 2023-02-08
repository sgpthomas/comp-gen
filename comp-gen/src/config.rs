use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
    pub dump_rules: bool,
    pub reuse_egraphs: bool,
    pub cd_filter: Option<f64>,
    pub require_all_vars: bool,
    pub phases: Vec<PhaseConfiguration>,
    pub scheduler: Option<RuleSchedulerOpt>,
    pub stats: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhaseConfiguration {
    pub name: String,
    pub cd: [Option<f64>; 2],
    pub ca: [Option<f64>; 2],
    pub fresh_egraph: Option<bool>,
    pub node_limit: Option<usize>,
    pub iter_limit: Option<usize>,
    pub disabled: Option<bool>,
}

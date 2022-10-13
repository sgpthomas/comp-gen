use serde::{Deserialize, Serialize};

/// A serializable configuration struct so that configurations can be loaded
/// from files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompilerConfiguration {
    pub total_node_limit: usize,
    pub timeout: u64,
    pub dry_run: bool,
    pub dump_rules: bool,
    pub reuse_egraphs: bool,
    pub cd_filter: Option<f64>,
    pub phases: Vec<PhaseConfiguration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhaseConfiguration {
    pub name: String,
    pub node_limit: Option<usize>,
    pub iter_limit: Option<usize>,
    pub cd: [Option<f64>; 2],
    pub ca: [Option<f64>; 2],
    pub disabled: Option<bool>,
}

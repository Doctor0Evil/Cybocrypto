use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    pub fixed_tick_ms: u32,
    pub lockstep_network: bool,
    pub record_rng_seed: bool,
    pub record_external_events: bool,
}

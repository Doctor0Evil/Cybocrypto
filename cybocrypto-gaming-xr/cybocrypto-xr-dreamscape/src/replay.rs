use crate::session_phoenix::{DecisionLogEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySeed {
    pub rng_seed: u64,
    pub initial_state_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayDescriptor {
    pub session_id: String,
    pub seed: ReplaySeed,
    pub decisions: Vec<DecisionLogEntry>, // ordered by seq
}

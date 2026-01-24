use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Core ALN context for any ledger-anchored operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnContext {
    pub network_id: String,
    pub chain_id: String,
    pub session_id: String,
    pub timestamp_ms: u64,
}

impl AlnContext {
    pub fn new(network_id: impl Into<String>, chain_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            network_id: network_id.into(),
            chain_id: chain_id.into(),
            session_id: session_id.into(),
            timestamp_ms: now.as_millis() as u64,
        }
    }
}

/// Minimal progress stamp for game/XR state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStamp {
    pub seq: u64,
    pub context: AlnContext,
}

/// Trait for a single deterministic progression step on any payload.
pub trait ProgressOnce<P> {
    type Error;

    fn progress_once(&mut self, input: P) -> Result<P, Self::Error>;
}

/// Trait for attaching state to a ledger (abstract over chain implementation).
pub trait AnchorToLedger {
    type CommitId;
    type Error;

    fn commit_state(&self, ctx: &AlnContext) -> Result<Self::CommitId, Self::Error>;
}

/// Marker trait for bioscale-safe operations.
pub trait BioscaleSafe {}

/// Helper to create a new ALN context for gaming/XR.
pub fn new_gaming_context(session_id: impl Into<String>) -> AlnContext {
    AlnContext::new("cybocrypto-gaming", "xr-dreamscape-chain", session_id)
}

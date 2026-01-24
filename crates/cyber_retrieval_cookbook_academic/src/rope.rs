use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{KsrTriple, PromptEnvelope};

/// Identifier for a NeuralRope (research session).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NeuralRopeId(pub String);

/// One segment in a NeuralRope, with explicit K/S/R deltas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRopeSegment {
    pub rope_id: NeuralRopeId,
    pub index: u32,
    pub envelope: PromptEnvelope,
    /// Delta in K/S/R contributed by this step (post-hoc or predicted).
    pub ksr_delta: KsrTriple,
    /// Cumulative K/S/R after this step.
    pub ksr_cumulative: KsrTriple,
    /// Estimated Risk-of-Harm after this step, 0–100 scale.
    pub roh_index: u8,
    /// Timestamp when this segment was logged.
    pub logged_at: DateTime<Utc>,
}

/// Full Rope: ordered list of segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRope {
    pub id: NeuralRopeId,
    pub segments: Vec<NeuralRopeSegment>,
}

impl NeuralRope {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: NeuralRopeId(id.into()),
            segments: Vec::new(),
        }
    }

    /// Append a segment; caller computes deltas and RoH.
    pub fn push_segment(&mut self, segment: NeuralRopeSegment) {
        self.segments.push(segment);
    }

    /// Get current cumulative K/S/R if present.
    pub fn current_ksr(&self) -> Option<KsrTriple> {
        self.segments.last().map(|s| s.ksr_cumulative)
    }

    /// Get current RoH index if present.
    pub fn current_roh(&self) -> Option<u8> {
        self.segments.last().map(|s| s.roh_index)
    }
}

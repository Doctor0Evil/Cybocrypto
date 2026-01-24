#![forbid(unsafe_code)]

pub mod intent;
pub mod envelope;
pub mod ksrs;
pub mod rope;
pub mod quiz_math;

/// Convenience re-exports for downstream crates.
pub use intent::RetrievalIntent;
pub use envelope::{PromptEnvelope, Domain, XrZone, AllowedCodeActions};
pub use ksrs::{KsrTriple, KSR_CEILING_DEFAULT};
pub use rope::{NeuralRopeId, NeuralRopeSegment, NeuralRope};
pub use quiz_math::{QuizResult, QuizScore};

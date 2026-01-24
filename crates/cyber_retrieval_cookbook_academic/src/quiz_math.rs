use serde::{Deserialize, Serialize};

use crate::KsrTriple;

/// Score returned by quiz_math across a retrieval batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizScore {
    /// 0–100 confidence in factual consistency across sources.
    pub consistency_score: u8,
    /// 0–100 confidence that units/constraints match DCM/HCI & XRGrid specs.
    pub constraint_score: u8,
    /// 0–100 estimated Risk-of-Harm if these facts drive code/templates.
    pub risk_score: u8,
}

impl QuizScore {
    pub fn new(consistency_score: u8, constraint_score: u8, risk_score: u8) -> Self {
        Self {
            consistency_score,
            constraint_score,
            risk_score,
        }
    }

    /// Sane defaults for an uninitialized quiz.
    pub fn zero() -> Self {
        Self {
            consistency_score: 0,
            constraint_score: 0,
            risk_score: 0,
        }
    }
}

/// Result of quiz_math validation, including a recommended K/S/R.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizResult {
    pub score: QuizScore,
    pub recommended_ksr: KsrTriple,
    /// Whether code-generation should be allowed for this batch.
    pub allow_code_synthesis: bool,
}

impl QuizResult {
    /// Simple gate: allow code if risk is low and consistency/constraints are high.
    pub fn decide(score: QuizScore) -> Self {
        let allow = score.risk_score <= 30
            && score.consistency_score >= 70
            && score.constraint_score >= 70;

        // Map scores to a rough K/S/R triple.
        let k = 0xD0 + (score.consistency_score / 16);
        let s = 0x70 + (score.constraint_score / 16);
        let r = 0x10 + (score.risk_score / 16);

        let recommended_ksr = KsrTriple::new(k, s, r);

        Self {
            score,
            recommended_ksr,
            allow_code_synthesis: allow,
        }
    }
}

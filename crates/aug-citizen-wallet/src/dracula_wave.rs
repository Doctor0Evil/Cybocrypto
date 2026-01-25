use serde::{Deserialize, Serialize};

/// Banding aligned with Psyche_Junky: 0.0–0.4 NORMAL, 0.4–0.7 MODERATE, 0.7–1.0 HIGH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PsychBand {
    Normal,
    Moderate,
    High,
}

/// Aggregate psych_risk vector for Cybocrypto flows.
/// All components are dimensionless and host-protective.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PsychRiskVector {
    pub physload: f32,   // 0.0–1.0
    pub cogload: f32,    // 0.0–1.0
    pub envrisk: f32,    // 0.0–1.0
    pub devstress: f32,  // 0.0–1.0
    pub scalar: f32,     // aggregate 0.0–1.0
    pub band: PsychBand, // NORMAL / MODERATE / HIGH
}

/// Dracula_Wave corridor state as seen from Cybocrypto.
/// This shapes UI pacing and review thresholds, not hardware.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DraculaWaveCorridor {
    pub opacity: f32,        // 0.0–1.0: higher → more shielding / simpler UI
    pub loop_hz: f32,        // logical update rate for prompts / offers
    pub intensity_scale: f32 // 0.0–1.0: how “strong” the UX may feel
}

/// High-level Dracula_Wave status exported into payment and policy engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraculaWaveMode {
    Idle,
    Active,
}

/// Snapshot used by Cybocrypto routers and policy engines.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DraculaWaveSnapshot {
    pub mode: DraculaWaveMode,
    pub corridor: DraculaWaveCorridor,
    pub psych: PsychRiskVector,
}

/// Wallet-facing configuration for how Dracula_Wave should modulate flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraculaWavePolicy {
    /// Maximum allowed psych_risk scalar before we force HUMAN_REVIEW for non-basic flows.
    pub max_scalar_for_auto: f32,
    /// When band is HIGH, require human review for any non-basic, non-stipend payments.
    pub require_review_in_high_band: bool,
    /// When ACTIVE, clamp maximum prompts per minute for this profile.
    pub max_prompts_per_minute_active: u32,
}

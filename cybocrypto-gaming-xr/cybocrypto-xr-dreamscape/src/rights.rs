use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AugmentationClass {
    None,
    Cybernetic,
    OrganicIntegrated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RightsTier {
    BaselineCitizen,       // full fundamental rights
    AugmentedCitizen,      // same as baseline, plus augmentation-specific safeguards
    SovereignOperator,     // governance / node operator role
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignRightsProfile {
    pub augmentation: AugmentationClass,
    pub tier: RightsTier,
    pub can_request_full_replay_export: bool,
    pub can_request_replay_redaction: bool,
    pub can_restrict_inference_on_xr_signals: bool,
}

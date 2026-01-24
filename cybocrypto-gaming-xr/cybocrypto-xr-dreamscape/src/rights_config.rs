use crate::rights::{AugmentationClass, RightsTier, SovereignRightsProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignRightsConfig {
    pub default_retention_days: u32,
    pub allow_cross_region_mirroring: bool,
    pub require_regulatory_justification_for_access: bool,
}

impl SovereignRightsConfig {
    pub fn profile_for(
        &self,
        augmentation: AugmentationClass,
        tier: RightsTier,
    ) -> SovereignRightsProfile {
        let base = SovereignRightsProfile {
            augmentation: augmentation.clone(),
            tier: tier.clone(),
            can_request_full_replay_export: true,
            can_request_replay_redaction: true,
            can_restrict_inference_on_xr_signals: true,
        };

        match tier {
            RightsTier::BaselineCitizen => base,
            RightsTier::AugmentedCitizen => base,
            RightsTier::SovereignOperator => SovereignRightsProfile {
                can_request_full_replay_export: true,
                can_request_replay_redaction: true,
                can_restrict_inference_on_xr_signals: true,
                ..base
            },
        }
    }
}

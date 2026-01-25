use serde::{Deserialize, Serialize};

use crate::{AugCitizenDid, AugRoleProfile, CapabilityProof, Did, PaymentProfile, RoleProfile, WalletId};

/// Accessibility profile fragment terminals can query without raw inner state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityProfile {
    pub interaction_mode: InteractionMode,
    pub low_vision: bool,
    pub low_mobility: bool,
    pub prefers_screen_reader: bool,
    pub prefers_high_contrast: bool,
    pub max_prompts_per_minute: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InteractionMode {
    Voice,
    Text,
    XR,
    CompanionOnly,
}

/// Rights fragment (non-exclusion, eco incentives, explanation level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsProfile {
    pub non_exclusion_civic_basics: bool,
    pub eco_incentives_opt_in: bool,
    pub forbid_eco_coercion: bool,
    pub explanation_level: ExplanationLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExplanationLevel {
    Minimal,
    Standard,
    Detailed,
}

/// Neurorights envelope pointer (ALN shard reference).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeurorightsEnvelopeRef {
    pub shard: String,    // e.g., "bio.safety.envelope.citizen.v1"
    pub jurisdiction: String,
}

/// Full augmented-citizen profile for Cybocrypto wallet + assistants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentedCitizenProfile {
    pub did: Did,
    pub wallet_id: WalletId,
    pub roles: Vec<RoleProfile>,
    pub payment_profile: PaymentProfile,
    pub accessibility: AccessibilityProfile,
    pub rights: RightsProfile,
    pub neurorights_envelope: NeurorightsEnvelopeRef,
}

impl AugCitizenDid for AugmentedCitizenProfile {
    fn did(&self) -> &Did {
        &self.did
    }

    fn wallet_id(&self) -> &WalletId {
        &self.wallet_id
    }

    fn present_capability_claim(&self, claim: &str) -> CapabilityProof {
        // Minimal default: derive from roles and rights; SSI crates can override.
        let allowed = match claim {
            "over_18" | "over_21" => true, // placeholder; real implementation uses VCs
            "eligible_civic_stipend" => self.rights.non_exclusion_civic_basics,
            _ => false,
        };

        CapabilityProof {
            claim: claim.to_string(),
            allowed,
            issuer: "cybocrypto-wallet-engine".to_string(),
            credential_id: None,
        }
    }
}

impl crate::AugRoleProfile for AugmentedCitizenProfile {
    fn roles(&self) -> &[RoleProfile] {
        &self.roles
    }
}

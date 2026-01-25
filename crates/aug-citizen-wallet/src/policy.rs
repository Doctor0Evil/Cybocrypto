use serde::{Deserialize, Serialize};

use crate::{AugmentedCitizenProfile, PaymentContext, PaymentDecision, PaymentType};

/// High-level payment request used by Cybocrypto agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub payment_type: PaymentType,
    pub context: PaymentContext,
    pub amount: u64,
    pub is_basic_service: bool,
}

/// Result of the wallet policy evaluation for a payment request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentPolicyResult {
    pub decision: PaymentDecision,
    pub explanation: String,
}

/// Core evaluator combining rights, roles, and payment profile.
pub fn evaluate_payment_request(
    profile: &AugmentedCitizenProfile,
    request: &PaymentRequest,
) -> PaymentPolicyResult {
    let decision = profile
        .payment_profile
        .is_allowed(request.payment_type, request.context, request.amount, request.is_basic_service);

    let explanation = match &decision {
        PaymentDecision::AllowAuto => {
            if request.is_basic_service && profile.rights.non_exclusion_civic_basics {
                "Allowed automatically: civic basic with non-exclusion guarantee.".to_string()
            } else {
                "Allowed automatically by payment policy.".to_string()
            }
        }
        PaymentDecision::RequireHumanReview(reason) => {
            format!("Requires human review: {:?}", reason)
        }
        PaymentDecision::Denied(reason) => {
            format!("Denied by wallet policy: {:?}", reason)
        }
    };

    PaymentPolicyResult { decision, explanation }
}

use serde::{Deserialize, Serialize};

/// Payment type primitives aligned with your civic stipend, accessibility credits, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentType {
    CivicStipend,
    AccessibilityCredit,
    DataDividend,
    CommunityCareToken,
}

/// Context tags for payments (civic basic, health, gaming extras, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentContext {
    CivicBasic,
    HealthAccess,
    AccessibilitySupport,
    Mobility,
    GamingExtras,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentPolicy {
    pub payment_type: PaymentType,
    pub allowed_contexts: Vec<PaymentContext>,
    pub non_exclusion: bool,       // true if must not be denied for civic basics
    pub max_auto_amount: u64,      // smallest common unit (e.g., cents)
    pub require_human_appeal_path: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProfile {
    pub policies: Vec<PaymentPolicy>,
}

impl PaymentProfile {
    pub fn policy_for(&self, payment_type: PaymentType) -> Option<&PaymentPolicy> {
        self.policies.iter().find(|p| p.payment_type == payment_type)
    }

    pub fn is_allowed(
        &self,
        payment_type: PaymentType,
        context: PaymentContext,
        amount: u64,
        is_basic_service: bool,
    ) -> PaymentDecision {
        if let Some(policy) = self.policy_for(payment_type) {
            if !policy.allowed_contexts.contains(&context) {
                return PaymentDecision::Denied(PaymentDenyReason::ContextNotAllowed);
            }

            if is_basic_service && policy.non_exclusion && amount <= policy.max_auto_amount {
                return PaymentDecision::AllowAuto;
            }

            if amount > policy.max_auto_amount {
                if policy.require_human_appeal_path {
                    return PaymentDecision::RequireHumanReview(PaymentReviewReason::HighAmount);
                } else {
                    return PaymentDecision::Denied(PaymentDenyReason::AmountTooHigh);
                }
            }

            PaymentDecision::AllowAuto
        } else {
            PaymentDecision::Denied(PaymentDenyReason::NoPolicy)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentDecision {
    AllowAuto,
    RequireHumanReview(PaymentReviewReason),
    Denied(PaymentDenyReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentReviewReason {
    HighAmount,
    RegulatedItem,
    RightsEnvelopeViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentDenyReason {
    NoPolicy,
    ContextNotAllowed,
    AmountTooHigh,
}

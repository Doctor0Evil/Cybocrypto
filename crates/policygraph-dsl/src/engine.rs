use aug_citizen_wallet::{
    evaluate_payment_request, AugmentedCitizenProfile, PaymentContext, PaymentDecision, PaymentRequest,
};
use crate::{CompiledPolicy, ConstraintKind, ContextKind, RemedyKind, RightKind};

/// Evaluation result combining wallet policy and graph-derived remedies.
#[derive(Debug, Clone)]
pub struct PolicyGraphEvaluation {
    pub decision: PaymentDecision,
    pub channel_remedy: Option<RemedyKind>,
    pub requires_explanation: bool,
}

/// Apply compiled policies to a payment request and AugmentedCitizenProfile.
pub fn evaluate_with_graph(
    profile: &AugmentedCitizenProfile,
    request: &PaymentRequest,
    policies: &[CompiledPolicy],
) -> PolicyGraphEvaluation {
    // First, use wallet-internal policy.
    let wallet_result = evaluate_payment_request(profile, request);

    // Then, apply graph-based constraints/remedies for the matching context.
    let context_kind = map_context_from_request(request.context);
    let mut channel_remedy = None;
    mut requires_explanation = false;

    for policy in policies {
        if let Some(right) = policy.right {
            match right {
                RightKind::NonExclusionCivicBasics => {
                    if request.is_basic_service && matches!(context_kind, Some(ContextKind::CivicBasic)) {
                        // If wallet denied, upgrade to human review rather than silent denial.
                        if matches!(wallet_result.decision, PaymentDecision::Denied(_)) {
                            channel_remedy = Some(RemedyKind::RedirectToClerk);
                            requires_explanation = true;
                        }
                    }
                }
                RightKind::AccessibilityFirst => {
                    if matches!(
                        context_kind,
                        Some(ContextKind::AccessibilitySupport) | Some(ContextKind::HealthAccess)
                    ) {
                        channel_remedy.get_or_insert(RemedyKind::RedirectToCompanion);
                    }
                }
                RightKind::ExplanationRequired => {
                    requires_explanation = true;
                }
            }
        }

        if let Some(ctx) = policy.context {
            if Some(ctx) != context_kind {
                continue;
            }
        }

        for c in &policy.constraints {
            match c {
                ConstraintKind::MaxAmountAuto(limit) => {
                    if request.amount > *limit {
                        requires_explanation = true;
                    }
                }
                ConstraintKind::RequireHumanAppeal => {
                    if matches!(wallet_result.decision, PaymentDecision::Denied(_)) {
                        channel_remedy.get_or_insert(RemedyKind::RedirectToClerk);
                    }
                }
            }
        }

        for r in &policy.remedies {
            channel_remedy.get_or_insert(*r);
        }
    }

    PolicyGraphEvaluation {
        decision: wallet_result.decision,
        channel_remedy,
        requires_explanation,
    }
}

fn map_context_from_request(ctx: PaymentContext) -> Option<ContextKind> {
    match ctx {
        PaymentContext::CivicBasic => Some(ContextKind::CivicBasic),
        PaymentContext::HealthAccess => Some(ContextKind::HealthAccess),
        PaymentContext::AccessibilitySupport => Some(ContextKind::AccessibilitySupport),
        PaymentContext::Mobility => Some(ContextKind::Mobility),
        PaymentContext::GamingExtras => Some(ContextKind::GamingExtras),
        PaymentContext::Other => None,
    }
}

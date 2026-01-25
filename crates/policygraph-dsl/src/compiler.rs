use crate::{CompiledPolicy, ConstraintKind, ContextKind, Node, NodeKind, PolicyGraphAst, RemedyKind, RightKind};
use aug_citizen_wallet::PaymentType;

/// Very small compiler from AST to a single CompiledPolicy.
/// In a full implementation, you would support many policies and richer matching.
pub fn compile_ast(ast: &PolicyGraphAst) -> Vec<CompiledPolicy> {
    let mut policies = Vec::new();

    // For simplicity, treat each Right node as root for a policy.
    for node in ast.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Right)) {
        let mut policy = CompiledPolicy {
            right: map_right(&node.label),
            context: None,
            constraints: Vec::new(),
            remedies: Vec::new(),
            payment_type: None,
        };

        // Walk edges from this node in a very simple way.
        for edge in ast.edges.iter().filter(|e| e.from.0 == node.id.0) {
            if let Some(target) = ast.nodes.iter().find(|n| n.id.0 == edge.to.0) {
                match target.kind {
                    NodeKind::Context => {
                        policy.context = map_context(&target.label);
                    }
                    NodeKind::Constraint => {
                        if let Some(c) = map_constraint(&target.label) {
                            policy.constraints.push(c);
                        }
                    }
                    NodeKind::Remedy => {
                        if let Some(r) = map_remedy(&target.label) {
                            policy.remedies.push(r);
                        }
                    }
                    NodeKind::Role => {
                        policy.payment_type = map_payment_type(&target.label);
                    }
                    NodeKind::Right => {}
                }
            }
        }

        policies.push(policy);
    }

    policies
}

fn map_right(label: &str) -> Option<RightKind> {
    match label {
        "non_exclusion_civic_basics" => Some(RightKind::NonExclusionCivicBasics),
        "accessibility_first" => Some(RightKind::AccessibilityFirst),
        "explanation_required" => Some(RightKind::ExplanationRequired),
        _ => None,
    }
}

fn map_context(label: &str) -> Option<ContextKind> {
    match label {
        "civic_basic" => Some(ContextKind::CivicBasic),
        "health_access" => Some(ContextKind::HealthAccess),
        "accessibility_support" => Some(ContextKind::AccessibilitySupport),
        "mobility" => Some(ContextKind::Mobility),
        "gaming_extras" => Some(ContextKind::GamingExtras),
        _ => None,
    }
}

fn map_constraint(label: &str) -> Option<ConstraintKind> {
    if let Some(rest) = label.strip_prefix("max_amount_auto_") {
        if let Ok(v) = rest.parse::<u64>() {
            return Some(ConstraintKind::MaxAmountAuto(v));
        }
    }
    match label {
        "require_human_appeal" => Some(ConstraintKind::RequireHumanAppeal),
        _ => None,
    }
}

fn map_remedy(label: &str) -> Option<RemedyKind> {
    match label {
        "redirect_to_clerk" => Some(RemedyKind::RedirectToClerk),
        "redirect_to_companion" => Some(RemedyKind::RedirectToCompanion),
        "provide_explanation" => Some(RemedyKind::ProvideExplanation),
        _ => None,
    }
}

fn map_payment_type(label: &str) -> Option<PaymentType> {
    match label {
        "civic_stipend" => Some(PaymentType::CivicStipend),
        "accessibility_credit" => Some(PaymentType::AccessibilityCredit),
        "data_dividend" => Some(PaymentType::DataDividend),
        "community_care_token" => Some(PaymentType::CommunityCareToken),
        _ => None,
    }
}

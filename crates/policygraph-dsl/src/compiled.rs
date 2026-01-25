use serde::{Deserialize, Serialize};

use aug_citizen_wallet::{PaymentContext, PaymentType};

/// Rights enum compiled from AST Right nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RightKind {
    NonExclusionCivicBasics,
    AccessibilityFirst,
    ExplanationRequired,
}

/// Context enum compiled from AST Context nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextKind {
    CivicBasic,
    HealthAccess,
    AccessibilitySupport,
    Mobility,
    GamingExtras,
}

/// Constraint enum compiled from AST Constraint nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConstraintKind {
    MaxAmountAuto(u64),
    RequireHumanAppeal,
}

/// Remedy enum compiled from AST Remedy nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemedyKind {
    RedirectToClerk,
    RedirectToCompanion,
    ProvideExplanation,
}

/// Compiled policy snippet used in the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPolicy {
    pub right: Option<RightKind>,
    pub context: Option<ContextKind>,
    pub constraints: Vec<ConstraintKind>,
    pub remedies: Vec<RemedyKind>,
    pub payment_type: Option<PaymentType>,
}

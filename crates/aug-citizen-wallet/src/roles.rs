use serde::{Deserialize, Serialize};

/// High-level roles across civic, mobility, health, and gaming domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleKind {
    Resident,
    Traveler,
    Patient,
    Gamer,
    Caregiver,
}

/// Role capability flags derived from rights and envelopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCapabilities {
    pub can_access_civic_basics: bool,
    pub can_receive_civic_stipend: bool,
    pub can_receive_accessibility_credits: bool,
    pub can_receive_data_dividends: bool,
    pub can_receive_care_tokens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleProfile {
    pub role: RoleKind,
    pub capabilities: RoleCapabilities,
}

/// Trait for role-aware profiles.
pub trait AugRoleProfile {
    fn roles(&self) -> &[RoleProfile];

    fn has_role(&self, role: RoleKind) -> bool {
        self.roles().iter().any(|r| r.role == role)
    }

    fn role_capabilities(&self, role: RoleKind) -> Option<&RoleCapabilities> {
        self.roles().iter().find(|r| r.role == role).map(|r| &r.capabilities)
    }
}

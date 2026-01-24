use crate::encoding::ReplayEncodingFormat;
use crate::replay::ReplayDescriptor;
use crate::rights::SovereignRightsProfile;
use crate::sovereign::{SovereignRegion, SovereignStorageTarget};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditPurpose {
    AntiCheatReplay,
    SafetyAnalysis,
    RegulatoryCompliance,
    UserAccessRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditVersion {
    pub schema_version: u16,
    pub codec_version: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAuditRecord {
    pub audit_id: String,
    pub session_id: String,

    pub purpose: AuditPurpose,
    pub version: AuditVersion,

    pub replay_descriptor: ReplayDescriptor,
    pub encoding_format: ReplayEncodingFormat,

    pub storage_target: SovereignStorageTarget,
    pub subject_rights: SovereignRightsProfile,

    pub created_at_ms: u64,
}

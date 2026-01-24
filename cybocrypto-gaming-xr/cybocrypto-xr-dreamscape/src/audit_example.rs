use crate::audit::{AuditPurpose, AuditVersion, SovereignAuditRecord};
use crate::encoding::{encode_replay, ReplayEncodingFormat};
use crate::frame_store::StoredFrameRecord;
use crate::replay::ReplayDescriptor;
use crate::rights::{AugmentationClass, RightsTier};
use crate::rights_config::SovereignRightsConfig;
use crate::sovereign::{SovereignRegion, StorageBackendKind, SovereignStorageTarget};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn build_audit_record_example(
    session_id: &str,
    descriptor: ReplayDescriptor,
    frames: &[StoredFrameRecord],
) -> Result<(SovereignAuditRecord, Vec<u8>), String> {
    let encoding_format = ReplayEncodingFormat::BinaryBincode;
    let encoded = encode_replay(&descriptor, frames, encoding_format.clone())?;

    let rights_config = SovereignRightsConfig {
        default_retention_days: 365,
        allow_cross_region_mirroring: false,
        require_regulatory_justification_for_access: true,
    };

    // Example: organically-integrated augmented-citizen, equal rights.
    let subject_rights = rights_config.profile_for(
        AugmentationClass::OrganicIntegrated,
        RightsTier::AugmentedCitizen,
    );

    let storage_target = SovereignStorageTarget {
        region: SovereignRegion::Custom("Offshore-Node-01".to_string()),
        backend: StorageBackendKind::AppendLog,
        endpoint_label: "audit-ledger-node-01".to_string(),
        bucket_or_db: "xr_audit".to_string(),
        collection: "replay_bundles".to_string(),
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let audit_record = SovereignAuditRecord {
        audit_id: format!("audit:{}:{}", session_id, now_ms),
        session_id: session_id.to_string(),
        purpose: AuditPurpose::RegulatoryCompliance,
        version: AuditVersion {
            schema_version: 1,
            codec_version: 1,
        },
        replay_descriptor: descriptor,
        encoding_format,
        storage_target,
        subject_rights,
        created_at_ms: now_ms,
    };

    Ok((audit_record, encoded))
}

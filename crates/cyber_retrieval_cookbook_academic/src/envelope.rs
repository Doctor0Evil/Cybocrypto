use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{KsrTriple, RetrievalIntent};

/// High-level domain tags for Cookbook routing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Domain {
    DcmHciDesign,
    XrGridPolicy,
    RustWiring,
    DidRegistry,
    AcademicKnowledge,
}

/// XR zone identifiers compatible with XR-Grid zoning.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum XrZone {
    Phoenix,
    SanJolla,
    Eco,
    Unknown,
}

/// Allowed code actions for this retrieval step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedCodeActions {
    /// Whether code generation is allowed at all.
    pub allow_code_synthesis: bool,
    /// Whether manifests or policy templates may be emitted.
    pub allow_manifest_templates: bool,
    /// Whether only retrieval/summary is allowed (no code).
    pub retrieval_only: bool,
}

/// Canonical PromptEnvelope for academic/cybernetic retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEnvelope {
    /// Stable trace id for this step (e.g., deterministic hash).
    pub trace_id: String,
    /// Human- or system-provided prompt text.
    pub prompt_text: String,
    /// Retrieval intent (governed enum).
    pub intent: RetrievalIntent,
    /// Cookbook domain routing tag.
    pub domain: Domain,
    /// XR zone tag for zoning/consent.
    pub xr_zone: XrZone,
    /// Knowledge/Social/Risk estimate prior to running tools.
    pub ksr_estimate: KsrTriple,
    /// Which code actions are permitted at this step.
    pub allowed_code_actions: AllowedCodeActions,
    /// Creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
}

impl PromptEnvelope {
    /// Convenience constructor.
    pub fn new(
        trace_id: impl Into<String>,
        prompt_text: impl Into<String>,
        intent: RetrievalIntent,
        domain: Domain,
        xr_zone: XrZone,
        ksr_estimate: KsrTriple,
        allowed_code_actions: AllowedCodeActions,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            prompt_text: prompt_text.into(),
            intent,
            domain,
            xr_zone,
            ksr_estimate,
            allowed_code_actions,
            created_at,
        }
    }
}

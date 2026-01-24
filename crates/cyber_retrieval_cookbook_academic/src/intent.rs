use serde::{Deserialize, Serialize};

/// Retrieval intents for the Cybernetic Cookbook academic lane.
/// These are first-class and govern which sources and depths are allowed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalIntent {
    /// General academic knowledge: papers, standards, reference docs.
    RetrieveKnowledgeAcademic,
    /// Scan for risks, red flags, or threat patterns in sources.
    ThreatScanAcademic,
    /// Retrieve device/manifest/policy material related to DCM/HCI/XR.
    RetrievePolicyDcmHci,
    /// Multi-step, research-grade Rope traversal for cybernetics topics.
    NeuralRopeResearchAcademic,
}

use crate::chat::{AiChatFrame, ChatSpeaker};
use crate::sovereign::{SovereignRegion, SovereignStorageTarget};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataResidencyClass {
    Exportable,      // can be replicated out of origin region
    RegionBound,     // must stay in declared region (e.g. EU)
    StrictLocal,     // must remain on local cluster only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignPolicy {
    pub residency: DataResidencyClass,
    pub retention_days: u32,
    pub allow_offline_export: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatSovereignEnvelope {
    pub frame: AiChatFrame,
    pub storage_target: SovereignStorageTarget,
    pub policy: SovereignPolicy,
}

impl AiChatSovereignEnvelope {
    pub fn new_region_bound(frame: AiChatFrame, target: SovereignStorageTarget) -> Self {
        Self {
            frame,
            storage_target: target,
            policy: SovereignPolicy {
                residency: DataResidencyClass::RegionBound,
                retention_days: 365,
                allow_offline_export: false,
            },
        }
    }

    pub fn speaker(&self) -> &ChatSpeaker {
        &self.frame.speaker
    }
}

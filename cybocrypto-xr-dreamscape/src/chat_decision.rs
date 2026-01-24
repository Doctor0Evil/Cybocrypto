use serde::{Deserialize, Serialize};

use crate::chat::{AiChatFrame, ChatSpeaker};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiDecisionKind {
    SuggestMove,
    AutoMove,
    DialogueChoice,
    WorldEdit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDecisionPayload {
    pub kind: AiDecisionKind,
    pub action_id: String,              // canonical game action
    pub parameters: serde_json::Value,  // small JSON args
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatDecision {
    pub seq: u64,                       // chat sequence in this session
    pub frame: AiChatFrame,             // full frame for local replay
    pub decision: Option<AiDecisionPayload>,
}

impl AiChatDecision {
    pub fn is_commit_critical(&self) -> bool {
        matches!(self.frame.speaker, ChatSpeaker::AiAgent | ChatSpeaker::Player)
            && self.decision.is_some()
    }
}

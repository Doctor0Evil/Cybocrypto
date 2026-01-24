use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatSpeaker {
    Player,
    Npc,
    System,
    AiAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatFrame {
    pub speaker: ChatSpeaker,
    pub identity_id: String,    // Bostrom/NeuroIdentity reference
    pub realm: String,          // XR world instance
    pub message: String,
    pub timestamp_ms: u64,
    pub correlation_id: String, // tie chat to game events
}

impl AiChatFrame {
    pub fn new(
        speaker: ChatSpeaker,
        identity_id: impl Into<String>,
        realm: impl Into<String>,
        message: impl Into<String>,
        correlation_id: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            speaker,
            identity_id: identity_id.into(),
            realm: realm.into(),
            message: message.into(),
            timestamp_ms: now.as_millis() as u64,
            correlation_id: correlation_id.into(),
        }
    }
}

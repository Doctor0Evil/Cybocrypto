use crate::chat::{AiChatFrame, ChatSpeaker, AiDecisionPayload, AiDecisionKind};
use crate::sovereign::{SovereignRegion, StorageBackendKind, SovereignStorageTarget};
use crate::sovereign_chat::AiChatSovereignEnvelope;
use serde_json::json;

/// Example: build a frame + envelope for an offshore-capable AI reply.
pub fn build_sovereign_ai_reply(identity_id: &str, realm: &str) -> AiChatSovereignEnvelope {
    let decision = AiDecisionPayload {
        kind: AiDecisionKind::SuggestMove,
        action_id: "move_to_portal".to_string(),
        parameters: json!({ "portal_id": "xr:phoenix:gate-01" }),
    };

    let base_frame = AiChatFrame::new_chat(
        ChatSpeaker::AiAgent,
        identity_id.to_string(),
        realm.to_string(),
        "Head to the Phoenix Gate to continue.",
        "ai-suggest-move",
    )
    .with_decision(decision);

    // Define an offshore or region-specific storage target under your control.
    let target = SovereignStorageTarget {
        region: SovereignRegion::Custom("Offshore-Node-01".to_string()),
        backend: StorageBackendKind::AppendLog,
        endpoint_label: "dreamscape-log-node-01".to_string(),
        bucket_or_db: "xr_events".to_string(),
        collection: "ai_chat_frames".to_string(),
    };

    AiChatSovereignEnvelope::new_region_bound(base_frame, target)
}

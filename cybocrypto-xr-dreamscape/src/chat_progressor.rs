use cybocrypto_aln_core::{AlnContext, HostBudget, ProgressStamp, Provenance};
use cybocrypto_aln_progressor_macros::aln_progressor;

use crate::chat::AiChatFrame;
use crate::chat_decision::{AiChatDecision, AiDecisionKind, AiDecisionPayload};

#[aln_progressor(
    input = "AiChatFrame",
    output = "AiChatDecision",
    anchor = "cybocrypto_biopay_ledger::BiopayLedger" // or a game-specific ledger
)]
pub struct AiChatChannel {
    pub id: String,
    pub aln_context: AlnContext,
    pub last_stamp: Option<ProgressStamp>,
    pub provenance: Option<Provenance>,
}

impl AiChatChannel {
    pub fn new(id: String, aln_context: AlnContext) -> Self {
        Self {
            id,
            aln_context,
            last_stamp: None,
            provenance: None,
        }
    }

    /// Domain-specific, deterministic evolution rule.
    /// Given a frame + host budget, choose an AI decision (or none).
    pub fn build_output(
        &self,
        input: &AiChatFrame,
        _host: &HostBudget,
    ) -> AiChatDecision {
        // Example: mark all AiAgent messages with a SuggestMove decision keyed by message text.
        let decision = if matches!(input.speaker, crate::chat::ChatSpeaker::AiAgent) {
            Some(AiDecisionPayload {
                kind: AiDecisionKind::SuggestMove,
                action_id: format!("move:{}", input.message),
                parameters: serde_json::json!({ "raw_text": input.message }),
            })
        } else {
            None
        };

        AiChatDecision {
            seq: input.timestamp_ms, // or use an external seq from your queue
            frame: input.clone(),
            decision,
        }
    }
}

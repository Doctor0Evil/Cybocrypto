use crate::chat::{AiChatFrame, ChatSpeaker};
use crate::chat_queue::AiChatQueue;
use cybocrypto_aln_core::{AlnContext, ProgressStamp};
use cybocrypto_game_session::{GameState, SessionError, XrGameSession};
use cybocrypto_neuro_identity::NeuroIdentity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixOnChain {
    #[aln(commit)]
    pub xp: u64,
    #[aln(commit)]
    pub level: u32,
    #[aln(commit)]
    pub last_chat_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixClientLocal {
    #[aln(local)]
    pub camera_pos: (f32, f32, f32),
    #[aln(local)]
    pub fov_deg: f32,
    #[aln(local)]
    pub chat_ui_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixEphemeral {
    #[aln(ephemeral)]
    pub temp_chat_buffer: Vec<AiChatFrame>,
    #[aln(ephemeral)]
    pub recent_actions: Vec<String>,
}

pub type PhoenixGameState = GameState<PhoenixOnChain, PhoenixClientLocal, PhoenixEphemeral>;

#[derive(Debug)]
pub struct PhoenixRuntime {
    pub session: XrGameSession<PhoenixOnChain, PhoenixClientLocal, PhoenixEphemeral>,
    pub chat_queue: AiChatQueue,
}

impl PhoenixRuntime {
    pub fn new(identity: NeuroIdentity) -> Self {
        let ctx: AlnContext = cybocrypto_aln_core::new_gaming_context("phoenix-session");
        let stamp = ProgressStamp { seq: 0, context: ctx };

        let on_chain = PhoenixOnChain {
            xp: 0,
            level: 1,
            last_chat_seq: 0,
        };

        let client_local = PhoenixClientLocal {
            camera_pos: (0.0, 1.6, 0.0),
            fov_deg: 90.0,
            chat_ui_open: false,
        };

        let ephemeral = PhoenixEphemeral {
            temp_chat_buffer: Vec::new(),
            recent_actions: Vec::new(),
        };

        let state = PhoenixGameState {
            on_chain,
            client_local,
            ephemeral,
            stamp,
        };

        let session = XrGameSession { identity, state };
        let chat_queue = AiChatQueue::new();

        Self { session, chat_queue }
    }

    pub fn emit_player_chat(&mut self, message: &str) {
        let id = self.session.identity.bostrom_id.clone();
        let frame = AiChatFrame::new(
            ChatSpeaker::Player,
            id,
            self.session.state.stamp.context.session_id.clone(),
            message.to_string(),
            "player-chat",
        );
        let seq = self.chat_queue.push(frame);
        self.session.state.on_chain.last_chat_seq = seq;
    }

    /// To be called by your AI/chat layer: push AI messages into the queue.
    pub fn emit_ai_chat(&mut self, message: &str) {
        let id = self.session.identity.bostrom_id.clone();
        let frame = AiChatFrame::new(
            ChatSpeaker::AiAgent,
            id,
            self.session.state.stamp.context.session_id.clone(),
            message.to_string(),
            "ai-chat",
        );
        let seq = self.chat_queue.push(frame);
        self.session.state.on_chain.last_chat_seq = seq;
    }

    /// Process all queued frames into the ephemeral buffer (one game tick).
    pub fn flush_chat_to_state(&mut self) -> Result<(), SessionError> {
        let frames = self.chat_queue.pop_all();
        for frame in frames {
            self.session
                .state
                .ephemeral
                .temp_chat_buffer
                .push(frame);
        }
        Ok(())
    }
}

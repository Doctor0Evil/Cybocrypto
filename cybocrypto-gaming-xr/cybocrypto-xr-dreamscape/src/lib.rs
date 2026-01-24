use cybocrypto_aln_core::{AlnContext, ProgressStamp};
use cybocrypto_neuro_identity::NeuroIdentity;
use cybocrypto_game_session::{GameState, XrGameSession, SessionError};
use serde::{Deserialize, Serialize};

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
    pub identity_id: String,
    pub realm: String,
    pub message: String,
    pub timestamp_ms: u64,
    pub correlation_id: String,
}

/// Attributes will be interpreted by a derive macro in a next step.
/// For now they only serve as design markers.
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

pub fn new_phoenix_session(identity: NeuroIdentity) -> XrGameSession<PhoenixOnChain, PhoenixClientLocal, PhoenixEphemeral> {
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

    XrGameSession { identity, state }
}

/// Example of feeding AiChatFrame into state and bumping last_chat_seq.
pub fn push_chat_frame<SOn, SCl>(
    session: &mut XrGameSession<PhoenixOnChain, PhoenixClientLocal, PhoenixEphemeral>,
    frame: AiChatFrame,
) -> Result<(), SessionError> {
    session.state.ephemeral.temp_chat_buffer.push(frame);
    session.state.on_chain.last_chat_seq += 1;
    Ok(())
}


/// Example high-level API to commit a session state.
pub fn commit_session_state<SOn, SCl, SEp>(
    session: &GameState<SOn, SCl, SEp>,
) -> Result<String, String>
where
    SOn: Serialize,
    SCl: Serialize,
    SEp: Serialize,
{
    let ctx: AlnContext = new_gaming_context("xr-session-commit");
    session.commit_state(&ctx)
}

/// Example: create an identity and a minimal XR session.
pub fn example_minimal_session() -> Result<(), SessionError> {
    use cybocrypto_neuro_identity::neuro_identity;

    let identity: NeuroIdentity = neuro_identity! {
        id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
        facets: {
            governance: { role: "player", level: 1 },
            bioscale: { organic_cpu: true, interface: "hci_neural_v1" },
            xr_profile: { avatar_id: "dreamscape:phoenix-node-01", world_realm: "phoenix_arena" }
        },
        constraints: [MinimalDisclosure, Revocable, QuantumReady]
    };

    #[allow(non_snake_case)]
    let mut session = xr_session! {
        name: PhoenixArenaSession,
        identity: identity,
        on_chain: {
            xp: u64 = 0,
            level: u32 = 1
        },
        client_local: {
            camera_pos: (f32, f32, f32) = (0.0, 1.6, 0.0),
            fov_deg: f32 = 90.0
        },
        ephemeral: {
            temp_chat_buffer: Vec<String> = Vec::new(),
            recent_actions: Vec<String> = Vec::new()
        }
    };

    session.enter()?;
    session.update(0.016)?;
    session.exit()?;

    Ok(())
}

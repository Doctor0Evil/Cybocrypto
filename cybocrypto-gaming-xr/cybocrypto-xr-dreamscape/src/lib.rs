use cybocrypto_aln_core::{new_gaming_context, AlnContext};
use cybocrypto_game_session::{GameState, SessionError, XrGameSession};
use cybocrypto_neuro_identity::{NeuroIdentity};
use serde::{Deserialize, Serialize};

/// XR session description macro.
#[macro_export]
macro_rules! xr_session {
    (
        name: $name:ident,
        identity: $identity:expr,
        on_chain: { $( $ofield:ident : $otype:ty = $oval:expr ),* $(,)? },
        client_local: { $( $cfield:ident : $ctype:ty = $cval:expr ),* $(,)? },
        ephemeral: { $( $efield:ident : $etype:ty = $eval:expr ),* $(,)? }
    ) => {{
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name OnChain {
            $( pub $ofield: $otype, )*
        }

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name ClientLocal {
            $( pub $cfield: $ctype, )*
        }

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name Ephemeral {
            $( pub $efield: $etype, )*
        }

        let on_chain = $name OnChain {
            $( $ofield: $oval, )*
        };

        let client_local = $name ClientLocal {
            $( $cfield: $cval, )*
        };

        let ephemeral = $name Ephemeral {
            $( $efield: $eval, )*
        };

        let ctx: cybocrypto_aln_core::AlnContext =
            cybocrypto_aln_core::new_gaming_context("xr-session");

        let state = cybocrypto_game_session::GameState {
            on_chain,
            client_local,
            ephemeral,
            stamp: cybocrypto_aln_core::ProgressStamp { seq: 0, context: ctx },
        };

        cybocrypto_game_session::XrGameSession {
            identity: $identity,
            state,
        }
    }};
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

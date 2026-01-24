use cybocrypto_aln_core::{AlnContext, AnchorToLedger, ProgressOnce, ProgressStamp};
use cybocrypto_neuro_identity::NeuroIdentity;
use serde::{Deserialize, Serialize};

/// Generic game state split for on-chain / off-chain / ephemeral.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState<O, C, E> {
    pub on_chain: O,
    pub client_local: C,
    pub ephemeral: E,
    pub stamp: ProgressStamp,
}

pub trait GameSession {
    type OnChain;
    type ClientLocal;
    type Ephemeral;
    type Error;

    fn enter(&mut self) -> Result<(), Self::Error>;
    fn update(&mut self, dt: f32) -> Result<(), Self::Error>;
    fn exit(&mut self) -> Result<(), Self::Error>;

    fn state(&self) -> &GameState<Self::OnChain, Self::ClientLocal, Self::Ephemeral>;
    fn state_mut(&mut self) -> &mut GameState<Self::OnChain, Self::ClientLocal, Self::Ephemeral>;
}

/// Macro to declare a ledger-anchored game state type.
#[macro_export]
macro_rules! game_state {
    (
        name: $name:ident,
        on_chain: { $( $ofield:ident : $otype:ty ),* $(,)? },
        client_local: { $( $cfield:ident : $ctype:ty ),* $(,)? },
        ephemeral: { $( $efield:ident : $etype:ty ),* $(,)? }
    ) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub on_chain: $name OnChain,
            pub client_local: $name ClientLocal,
            pub ephemeral: $name Ephemeral,
            pub stamp: cybocrypto_aln_core::ProgressStamp,
        }

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
    };
}

/// Simple ledger anchor: serialize to JSON and pretend to commit.
impl<O, C, E> AnchorToLedger for GameState<O, C, E>
where
    O: serde::Serialize,
    C: serde::Serialize,
    E: serde::Serialize,
{
    type CommitId = String;
    type Error = String;

    fn commit_state(&self, ctx: &AlnContext) -> Result<Self::CommitId, Self::Error> {
        let payload = serde_json::to_string(self).map_err(|e| e.to_string())?;
        // In a real implementation, this is where you integrate a Rust blockchain client.[web:16][web:17][web:20]
        let commit_id = format!(
            "commit:{}:{}:{}",
            ctx.network_id, ctx.chain_id, self.stamp.seq
        );
        let _ = payload; // placeholder to keep clippy happy
        Ok(commit_id)
    }
}

/// Example session error type.
#[derive(Debug)]
pub enum SessionError {
    Generic(String),
}

/// Basic XR game session using a NeuroIdentity.
pub struct XrGameSession<SOn, SCl, SEp> {
    pub identity: NeuroIdentity,
    pub state: GameState<SOn, SCl, SEp>,
}

impl<SOn, SCl, SEp> GameSession for XrGameSession<SOn, SCl, SEp> {
    type OnChain = SOn;
    type ClientLocal = SCl;
    type Ephemeral = SEp;
    type Error = SessionError;

    fn enter(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update(&mut self, _dt: f32) -> Result<(), Self::Error> {
        self.state.stamp.seq += 1;
        Ok(())
    }

    fn exit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn state(&self) -> &GameState<Self::OnChain, Self::ClientLocal, Self::Ephemeral> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut GameState<Self::OnChain, Self::ClientLocal, Self::Ephemeral> {
        &mut self.state
    }
}

impl<SOn, SCl, SEp> ProgressOnce<GameState<SOn, SCl, SEp>> for XrGameSession<SOn, SCl, SEp> {
    type Error = SessionError;

    fn progress_once(
        &mut self,
        input: GameState<SOn, SCl, SEp>,
    ) -> Result<GameState<SOn, SCl, SEp>, Self::Error> {
        self.state = input;
        self.update(0.0)?;
        Ok(self.state.clone())
    }
}

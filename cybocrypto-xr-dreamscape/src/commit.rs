use cybocrypto_aln_core::{AlnContext, AnchorToLedger};
use crate::session_phoenix::{PhoenixGameState, PhoenixOnChain};

pub fn commit_phoenix_for_replay(
    state: &PhoenixGameState,
    ctx: &AlnContext,
) -> Result<String, String> {
    let commit_on_chain = state.on_chain.to_commit_view();

    // Minimal payload: only commit-critical game progression.
    let payload = serde_json::to_string(&commit_on_chain).map_err(|e| e.to_string())?;

    // In a real integration, this payload feeds your ledger or ALN progressor.
    let _ = payload;

    state.commit_state(ctx)
}

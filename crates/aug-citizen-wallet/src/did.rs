use serde::{Deserialize, Serialize};

/// Minimal DID + wallet handle abstraction for Cybocrypto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did {
    pub method: String,   // e.g., "did:web", "did:iota"
    pub id: String,       // method-specific identifier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletId {
    pub network: String,  // e.g., "cybocrypto-mainnet", "phoenix-testnet"
    pub address: String,  // on-ledger address compatible with Bostrom / Cybocrypto
}

/// Base trait for DID lifecycle and VC presentation hooks.
pub trait AugCitizenDid {
    fn did(&self) -> &Did;
    fn wallet_id(&self) -> &WalletId;

    /// Present a minimal yes/no capability proof for a given claim,
    /// such as "over_18", "over_21", "eligible_civic_stipend".
    fn present_capability_claim(&self, claim: &str) -> CapabilityProof;
}

/// Minimal capability proof envelope.
/// Concrete VC/SSI crates can implement the actual proof wiring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProof {
    pub claim: String,
    pub allowed: bool,
    pub issuer: String,
    pub credential_id: Option<String>,
}

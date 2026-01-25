use serde::{Serialize, Deserialize};

pub type HexStamp = [u8; 8];          // 64-bit hex-stamp, printed as 0xC0F2...
pub type RopeSeqNo = u64;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct NeuralRopeId(pub [u8; 32]);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KerVector {
    pub knowledge_factor: f32, // KF
    pub risk_of_harm: f32,     // RoH
    pub cybostate_factor: f32, // CSF
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcoFootprint {
    pub latency_ms: u32,
    pub energy_millijoules: u32,
    pub carbon_band: u8, // 0..3 = {low, medium, high, extreme}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceTriplet {
    pub superchair_id: String,
    pub council_shard: String,  // e.g. "governance.chat.website.v1"
    pub proposer_shard: String, // e.g. "asset.chat.stake.v1"
}

// Minimal identity types; you will plug in your DID/Bostrom crates here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Did(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlnShardRef(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BostromAddress(pub String);

// Neurorights envelope reference (e.g. bio.safety.envelope.citizen.v1)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeurorightsEnvelopeRef(pub String);

// Cookbook + SwarmNet references
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwarmTaskNodeId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookbookPlaybookId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RopeStepKind {
    Normal,
    Repair,
    Retry,
    Rollback,
    Snapshot,
    Finalize,
}

// Core block payload: one neurorights-governed PromptEnvelope hop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeurorightsBoundPromptEnvelope {
    pub rope_id: NeuralRopeId,
    pub rope_seq_no: RopeSeqNo,

    // Identity / authorship
    pub did: Did,
    pub aln_context: AlnShardRef,
    pub bostrom_addr: BostromAddress,

    // Neurorights + governance
    pub neurorights_profile: NeurorightsEnvelopeRef,
    pub governance: GovernanceTriplet,
    pub ker: KerVector,

    // Execution context
    pub swarmnet_step: SwarmTaskNodeId,
    pub cookbook_playbook: CookbookPlaybookId,
    pub eco_impact: EcoFootprint,
    pub step_kind: RopeStepKind,

    // Linking
    pub prev_hex_stamp: HexStamp,
    pub this_hex_stamp: HexStamp,
}

// Rope-level event wrapper (allows future variants if needed).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RopeBlock {
    PromptHop(NeurorightsBoundPromptEnvelope),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralRope {
    pub id: NeuralRopeId,
    pub opened_at_unix_ms: u64,
    pub closed_at_unix_ms: Option<u64>,
    pub blocks: Vec<RopeBlock>,
}

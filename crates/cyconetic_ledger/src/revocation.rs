use serde::{Serialize, Deserialize};
use crate::{NeuralRopeId, RopeSeqNo, HexStamp, RopeBlock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RopeRevocationReason {
    UserRequested,
    JurisdictionOrder,
    PolicyUpgrade,
    SafetyViolation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RopeRevocationEvent {
    pub rope_id: NeuralRopeId,
    pub rope_seq_no: RopeSeqNo,
    pub prev_hex_stamp: HexStamp,
    pub this_hex_stamp: HexStamp,
    pub reason: RopeRevocationReason,
    pub jurisdiction: String,
    pub is_freeze: bool,
    pub is_delete_hint: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GovernanceBlock {
    RopeRevocation(RopeRevocationEvent),
}

// You can add GovernanceBlock as another variant of RopeBlock if desired.
impl From<GovernanceBlock> for RopeBlock {
    fn from(gb: GovernanceBlock) -> Self {
        match gb {
            GovernanceBlock::RopeRevocation(ev) => {
                // You could wrap this in a dedicated enum variant if you prefer.
                // For now we serialize as a prompt-like hop with special step_kind upstream.
                // Or you can add RopeBlock::Governance(GovernanceBlock) in the main enum.
                // Here we assume RopeBlock has been extended accordingly.
                RopeBlock::Governance(gb)
            }
        }
    }
}

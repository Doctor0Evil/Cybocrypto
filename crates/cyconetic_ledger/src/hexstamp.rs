use crate::{HexStamp, KerVector, GovernanceTriplet, RopeSeqNo};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HexStampInputV1<'a> {
    pub prev_hex_stamp: HexStamp,
    pub rope_seq_no: RopeSeqNo,
    pub canonical_envelope_bytes: &'a [u8],
    pub ker: &'a KerVector,
    pub governance: &'a GovernanceTriplet,
}

fn fold_bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut acc: u64 = 0;
    for (i, b) in bytes.iter().enumerate() {
        let shift = ((i % 8) * 8) as u32;
        let part = (*b as u64) << shift;
        acc = acc.wrapping_add(part ^ ((acc << 5) | (acc >> 59)));
    }
    acc
}

/// Deterministic, versioned hex-stamp.
/// Architecture-level collision resistance is provided by (rope_seq_no, DID, ALN, Bostrom, governance).
pub fn compute_hex_stamp_v1(input: &HexStampInputV1<'_>) -> HexStamp {
    let mut buf = Vec::with_capacity(
        8 + 8 + input.canonical_envelope_bytes.len() + std::mem::size_of::<KerVector>()
            + std::mem::size_of::<GovernanceTriplet>(),
    );

    buf.extend_from_slice(&input.prev_hex_stamp);
    buf.extend_from_slice(&input.rope_seq_no.to_le_bytes());

    buf.extend_from_slice(input.canonical_envelope_bytes);

    buf.extend_from_slice(&input.ker.knowledge_factor.to_le_bytes());
    buf.extend_from_slice(&input.ker.risk_of_harm.to_le_bytes());
    buf.extend_from_slice(&input.ker.cybostate_factor.to_le_bytes());

    buf.extend_from_slice(input.governance.superchair_id.as_bytes());
    buf.extend_from_slice(input.governance.council_shard.as_bytes());
    buf.extend_from_slice(input.governance.proposer_shard.as_bytes());

    let folded = fold_bytes_to_u64(&buf);
    folded.to_le_bytes()
}

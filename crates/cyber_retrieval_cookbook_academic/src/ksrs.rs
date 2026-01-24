use serde::{Deserialize, Serialize};

/// Knowledge / Social / Risk triple for a retrieval step or Rope segment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KsrTriple {
    pub knowledge: u8,
    pub social: u8,
    pub risk: u8,
}

impl KsrTriple {
    pub const fn new(knowledge: u8, social: u8, risk: u8) -> Self {
        Self {
            knowledge,
            social,
            risk,
        }
    }
}

/// Default K/S/R ceiling for academic retrieval (RoH <= 0.3).
pub const KSR_CEILING_DEFAULT: KsrTriple = KsrTriple {
    knowledge: 0xE2,
    social: 0x78,
    risk: 0x27,
};

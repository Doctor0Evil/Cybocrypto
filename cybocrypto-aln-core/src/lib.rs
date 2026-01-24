#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Host-level biophysical budget, aligned with existing HostBudget patterns.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostBudget {
    pub daily_energy_joules: f64,
    pub remaining_energy_joules: f64,
    pub daily_protein_grams: f64,
    pub remaining_protein_grams: f64,
}

/// Short hex tag + description, reused for evidence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub short_hex: &'static str,
    pub description: &'static str,
}

/// Ten-sequence biophysical evidence bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub sequences: [EvidenceTag; 10],
}

/// Default 10-sequence biophysical evidence, identical pattern to your bioscale store.
pub const DEFAULT_BIOPHYS_EVIDENCE: EvidenceBundle = EvidenceBundle {
    sequences: [
        EvidenceTag {
            short_hex: "a1f3c9b2",
            description: "Resting metabolic rate and ATP turnover in human tissue.",
        },
        EvidenceTag {
            short_hex: "4be79d01",
            description: "Mitochondrial oxidative phosphorylation efficiency.",
        },
        EvidenceTag {
            short_hex: "9cd4a7e8",
            description: "Protein synthesis cost per amino acid.",
        },
        EvidenceTag {
            short_hex: "2f8c6b44",
            description: "Thermoregulatory limits for safe core temperature elevation.",
        },
        EvidenceTag {
            short_hex: "7e1da2ff",
            description: "Peripheral circulation adaptation under metabolic load.",
        },
        EvidenceTag {
            short_hex: "5b93e0c3",
            description: "Neurovascular coupling constraints.",
        },
        EvidenceTag {
            short_hex: "d0174aac",
            description: "Safe duty cycles in cortical tissue.",
        },
        EvidenceTag {
            short_hex: "6ac2f9d9",
            description: "ML workload energy profiles on neuromorphic hardware.",
        },
        EvidenceTag {
            short_hex: "c4e61b20",
            description: "Protein turnover kinetics in neural tissue.",
        },
        EvidenceTag {
            short_hex: "8f09d5ee",
            description: "Inflammation/pain thresholds for reversible interventions.",
        },
    ],
};

/// ALN context for the transaction or progressor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlnContext {
    pub jurisdiction_code: String, // e.g., "US-AZ", "EU-IE"
    pub policy_capsule_id: String, // references Globe lattice capsule.[file:96]
}

/// One cryptographic evolution step (no Blake).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgressStamp {
    pub step_id: u64,
    pub prev_hash: Option<Vec<u8>>,
    pub new_hash: Vec<u8>,
    pub timestamp: SystemTime,
}

/// DID/Bostrom/ALN provenance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub did: String,
    pub bostrom_address: String,
    pub aln_context: AlnContext,
}

/// Trait for single-step I/O evolution.
pub trait ProgressOnce {
    type Input;
    type Output;
    type Error;

    fn progress_once(
        &mut self,
        input: Self::Input,
        host: &HostBudget,
    ) -> Result<(Self::Output, ProgressStamp), Self::Error>;
}

/// Trait for ledger anchoring, designed to map onto state-machine style blockchain crates.[web:118][web:129]
pub trait AnchorToLedger {
    type AnchorError;

    fn anchor_to_ledger(
        &self,
        stamp: &ProgressStamp,
        evidence: &EvidenceBundle,
    ) -> Result<(), Self::AnchorError>;
}

/// Trait for identity/provenance guarantees, to be backed by DID + Bostrom address proofs.[file:96]
pub trait TraceProvenance {
    type ProvenanceError;

    fn attach_provenance(&mut self, provenance: Provenance) -> Result<(), Self::ProvenanceError>;

    fn verify_provenance(&self) -> Result<&Provenance, Self::ProvenanceError>;
}

/// Marker for bioscale-safe operations.
pub trait BioscaleSafe {
    fn is_bioscale_safe(&self, host: &HostBudget) -> bool;
}

/// Macro entry stub; real proc-macro lives in a sibling crate.
/// This keeps this crate `no_proc_macro` so it can be used in WASM.
#[macro_export]
macro_rules! aln_progressor {
    (
        name: $name:ident,
        input: $input:ty,
        output: $output:ty,
        anchors: [$($anchor:ty),* $(,)?],
        guarantees: [$($guarantee:ident),* $(,)?]
    ) => {
        // Expanded by proc-macro crate `cybocrypto-aln-progressor-macros`.
        compile_error!("aln_progressor! must be expanded by the proc-macro crate; \
                       ensure `cybocrypto-aln-progressor-macros` is in your dependencies.");
    };
}

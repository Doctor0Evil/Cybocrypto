#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use cybocrypto_aln_core::{
    AlnContext, BioscaleSafe, EvidenceBundle, HostBudget, ProgressOnce, ProgressStamp,
    Provenance, TraceProvenance, DEFAULT_BIOPHYS_EVIDENCE,
};

/// Supported card networks.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CardNetwork {
    Amex,
    Discover,
    Mastercard,
    Visa,
}

/// Supported terminal vendors/platforms.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerminalVendor {
    Verifone,
    Clover,
    QuickTrip,
    Topaz,
    Other(&'static str),
}

/// Merchants like AMPM with CoreMark-backed inventory hints.[web:133][web:136]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerchantProfile {
    pub merchant_id: String,
    pub legal_name: String,
    pub jurisdiction: String,
    pub coremark_score: f32,
}

/// Simple biophysical MOP description for a payment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiophysicalMop {
    pub host_budget_snapshot: HostBudget,
    pub metabolic_delta_joules: f64,
    pub protein_delta_grams: f64,
    pub thermic_delta_celsius: f32,
}

/// Basic payment request type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayRequest {
    pub card_network: CardNetwork,
    pub terminal_vendor: TerminalVendor,
    pub merchant: MerchantProfile,
    pub amount_minor_units: u64,
    pub currency: String,
    pub host_mop: BiophysicalMop,
    pub created_at: SystemTime,
}

/// Payment decision and bioscale evaluation outcome.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayDecision {
    pub approved: bool,
    pub decline_reason: Option<String>,
    pub bioscale_safe: bool,
    pub projected_completion: Option<SystemTime>,
}

/// State for a single biopay progressor (e.g., per-session).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayProgressor {
    pub id: String,
    pub aln_context: AlnContext,
    pub provenance: Option<Provenance>,
    pub last_stamp: Option<ProgressStamp>,
}

#[derive(Debug)]
pub enum BiopayError {
    BioscaleViolation(String),
    ProvenanceMissing,
}

impl BioscaleSafe for BiopayRequest {
    fn is_bioscale_safe(&self, host: &HostBudget) -> bool {
        let projected_energy = self.host_mop.metabolic_delta_joules
            + (host.daily_energy_joules - host.remaining_energy_joules);
        projected_energy <= host.daily_energy_joules * 1.05
            && self.host_mop.thermic_delta_celsius <= 0.5
    }
}

impl BiopayProgressor {
    pub fn new(id: String, aln_context: AlnContext) -> Self {
        Self {
            id,
            aln_context,
            provenance: None,
            last_stamp: None,
        }
    }
}

impl TraceProvenance for BiopayProgressor {
    type ProvenanceError = BiopayError;

    fn attach_provenance(&mut self, provenance: Provenance) -> Result<(), Self::ProvenanceError> {
        self.provenance = Some(provenance);
        Ok(())
    }

    fn verify_provenance(&self) -> Result<&Provenance, Self::ProvenanceError> {
        self.provenance.as_ref().ok_or(BiopayError::ProvenanceMissing)
    }
}

impl ProgressOnce for BiopayProgressor {
    type Input = BiopayRequest;
    type Output = BiopayDecision;
    type Error = BiopayError;

    fn progress_once(
        &mut self,
        input: Self::Input,
        host: &HostBudget,
    ) -> Result<(Self::Output, ProgressStamp), Self::Error> {
        if !input.is_bioscale_safe(host) {
            return Err(BiopayError::BioscaleViolation(
                "Host biophysical budget exceeded by payment route".into(),
            ));
        }

        let stamp = ProgressStamp {
            step_id: self
                .last_stamp
                .as_ref()
                .map(|s| s.step_id + 1)
                .unwrap_or(0),
            prev_hash: self.last_stamp.as_ref().map(|s| s.new_hash.clone()),
            new_hash: vec![0u8; 32], // placeholder; real implementation uses approved hash crate.
            timestamp: SystemTime::now(),
        };

        self.last_stamp = Some(stamp.clone());

        let decision = BiopayDecision {
            approved: true,
            decline_reason: None,
            bioscale_safe: true,
            projected_completion: Some(input.created_at + Duration::from_secs(5)),
        };

        Ok((decision, stamp))
    }
}

/// Anchoring biopay actions into a ledger-aware metadata struct; actual ledger impl lives downstream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayLedgerAnchor {
    pub stamp: ProgressStamp,
    pub evidence: EvidenceBundle,
    pub merchant_id: String,
    pub amount_minor_units: u64,
    pub card_network: CardNetwork,
}

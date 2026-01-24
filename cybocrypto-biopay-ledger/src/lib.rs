#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use cybocrypto_aln_core::{AnchorToLedger, EvidenceBundle, ProgressStamp};
use cybocrypto_biopay_core::{BiopayLedgerAnchor, CardNetwork};

/// Simple account identifier (can wrap Bostrom/ALN DIDs).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

/// Balance state machine compatible with Substrate-like ledgers.[web:129][web:134]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayLedgerState {
    pub balances: HashMap<AccountId, u128>,
}

#[derive(Debug)]
pub enum LedgerError {
    InsufficientFunds,
    Overflow,
    AnchorFailure(String),
}

impl BiopayLedgerState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    pub fn credit(&mut self, who: &AccountId, amount: u128) -> Result<(), LedgerError> {
        let entry = self.balances.entry(who.clone()).or_insert(0);
        *entry = entry
            .checked_add(amount)
            .ok_or(LedgerError::Overflow)?;
        Ok(())
    }

    pub fn debit(&mut self, who: &AccountId, amount: u128) -> Result<(), LedgerError> {
        let entry = self.balances.entry(who.clone()).or_insert(0);
        if *entry < amount {
            return Err(LedgerError::InsufficientFunds);
        }
        *entry -= amount;
        Ok(())
    }
}

/// Anchor implementation: converts a BiopayLedgerAnchor into internal bookkeeping.
/// In a real chain, this maps to a pallet storage write or Cosmos/CometBFT ABCI commit.[web:118]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiopayLedger {
    pub state: BiopayLedgerState,
    pub anchors: Vec<BiopayLedgerAnchor>,
}

impl BiopayLedger {
    pub fn new() -> Self {
        Self {
            state: BiopayLedgerState::new(),
            anchors: Vec::new(),
        }
    }
}

impl AnchorToLedger for BiopayLedger {
    type AnchorError = LedgerError;

    fn anchor_to_ledger(
        &self,
        _stamp: &ProgressStamp,
        _evidence: &EvidenceBundle,
    ) -> Result<(), Self::AnchorError> {
        // In a real integration, verify and persist to the underlying chain.
        Ok(())
    }
}

impl BiopayLedger {
    pub fn record_anchor(
        &mut self,
        anchor: BiopayLedgerAnchor,
        from: &AccountId,
        to: &AccountId,
    ) -> Result<(), LedgerError> {
        let amount = anchor.amount_minor_units as u128;
        self.debit(from, amount)?;
        self.credit(to, amount)?;
        self.anchors.push(anchor);
        Ok(())
    }

    pub fn network_fee_account(network: CardNetwork) -> AccountId {
        AccountId(format!("network:{:?}", network))
    }
}

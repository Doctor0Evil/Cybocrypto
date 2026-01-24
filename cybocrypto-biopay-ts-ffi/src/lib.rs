#![forbid(unsafe_code)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::time::SystemTime;

use cybocrypto_aln_core::{AlnContext, HostBudget, Provenance};
use cybocrypto_biopay_core::{
    BiopayProgressor, BiopayRequest, BiophysicalMop, CardNetwork, MerchantProfile, TerminalVendor,
};

/// Simple opaque pointer handle for TS.
#[repr(C)]
pub struct BiopayProgressorHandle {
    inner: BiopayProgressor,
}

#[no_mangle]
pub extern "C" fn biopay_progressor_new(
    id: *const c_char,
    jurisdiction: *const c_char,
    policy_capsule: *const c_char,
) -> *mut BiopayProgressorHandle {
    if id.is_null() || jurisdiction.is_null() || policy_capsule.is_null() {
        return ptr::null_mut();
    }
    let id_str = unsafe { CStr::from_ptr(id) }.to_string_lossy().into_owned();
    let j_str = unsafe { CStr::from_ptr(jurisdiction) }
        .to_string_lossy()
        .into_owned();
    let p_str = unsafe { CStr::from_ptr(policy_capsule) }
        .to_string_lossy()
        .into_owned();

    let aln = AlnContext {
        jurisdiction_code: j_str,
        policy_capsule_id: p_str,
    };

    let inner = BiopayProgressor::new(id_str, aln);
    Box::into_raw(Box::new(BiopayProgressorHandle { inner }))
}

#[no_mangle]
pub extern "C" fn biopay_progressor_free(handle: *mut BiopayProgressorHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(handle);
    }
}

/// Minimal TS-facing payment call: host budget + bio MOP + merchant, network, terminal.
#[no_mangle]
pub extern "C" fn biopay_progress_once_simple(
    handle: *mut BiopayProgressorHandle,
    host_daily_energy_joules: f64,
    host_remaining_energy_joules: f64,
    host_daily_protein_grams: f64,
    host_remaining_protein_grams: f64,
    metabolic_delta_joules: f64,
    protein_delta_grams: f64,
    thermic_delta_celsius: f32,
    amount_minor_units: u64,
    currency: *const c_char,
    merchant_id: *const c_char,
    merchant_name: *const c_char,
    merchant_jurisdiction: *const c_char,
    network_code: u32, // 0=Amex,1=Discover,2=Mastercard,3=Visa
    terminal_code: u32, // 0=Verifone,1=Clover,2=QuickTrip,3=Topaz,4=Other
) -> bool {
    if handle.is_null()
        || currency.is_null()
        || merchant_id.is_null()
        || merchant_name.is_null()
        || merchant_jurisdiction.is_null()
    {
        return false;
    }

    let h = unsafe { &mut *handle };

    let currency_str = unsafe { CStr::from_ptr(currency) }
        .to_string_lossy()
        .into_owned();
    let merch_id_str = unsafe { CStr::from_ptr(merchant_id) }
        .to_string_lossy()
        .into_owned();
    let merch_name_str = unsafe { CStr::from_ptr(merchant_name) }
        .to_string_lossy()
        .into_owned();
    let merch_j_str = unsafe { CStr::from_ptr(merchant_jurisdiction) }
        .to_string_lossy()
        .into_owned();

    let host = HostBudget {
        daily_energy_joules: host_daily_energy_joules,
        remaining_energy_joules: host_remaining_energy_joules,
        daily_protein_grams: host_daily_protein_grams,
        remaining_protein_grams: host_remaining_protein_grams,
    };

    let mop = BiophysicalMop {
        host_budget_snapshot: host.clone(),
        metabolic_delta_joules,
        protein_delta_grams,
        thermic_delta_celsius,
    };

    let network = match network_code {
        0 => CardNetwork::Amex,
        1 => CardNetwork::Discover,
        2 => CardNetwork::Mastercard,
        _ => CardNetwork::Visa,
    };

    let terminal = match terminal_code {
        0 => TerminalVendor::Verifone,
        1 => TerminalVendor::Clover,
        2 => TerminalVendor::QuickTrip,
        3 => TerminalVendor::Topaz,
        _ => TerminalVendor::Other("Other"),
    };

    let merchant = MerchantProfile {
        merchant_id: merch_id_str,
        legal_name: merch_name_str,
        jurisdiction: merch_j_str,
        coremark_score: 1.0, // can be populated from CoreMark device profiles.
    };

    let request = BiopayRequest {
        card_network: network,
        terminal_vendor: terminal,
        merchant,
        amount_minor_units,
        currency: currency_str,
        host_mop: mop,
        created_at: SystemTime::now(),
    };

    match h.inner.progress_once(request, &host) {
        Ok((decision, _stamp)) => decision.approved && decision.bioscale_safe,
        Err(_) => false,
    }
}

/// Optional function to attach a DID/Bostrom provenance from TS layer.
#[no_mangle]
pub extern "C" fn biopay_progressor_attach_provenance(
    handle: *mut BiopayProgressorHandle,
    did: *const c_char,
    bostrom_addr: *const c_char,
) -> bool {
    if handle.is_null() || did.is_null() || bostrom_addr.is_null() {
        return false;
    }
    let h = unsafe { &mut *handle };
    let did_str = unsafe { CStr::from_ptr(did) }
        .to_string_lossy()
        .into_owned();
    let bostrom_str = unsafe { CStr::from_ptr(bostrom_addr) }
        .to_string_lossy()
        .into_owned();

    let provenance = Provenance {
        did: did_str,
        bostrom_address: bostrom_str,
        aln_context: h.inner.aln_context.clone(),
    };

    h.inner.attach_provenance(provenance).is_ok()
}

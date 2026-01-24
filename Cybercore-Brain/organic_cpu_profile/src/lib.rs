//! OrganicCpuProfile core traits and types.
//!
//! This crate defines:
//! - BrainSpecs and HostBudget (20-W envelope).
//! - Multimodal telemetry types and LifeforceState.
//! - Corridor budgets and QuantumRecedingEnvelope step-safety.
//! - PsychRisk, PDR, TelemetricalOsteosis caps.
//! - OrganicCpuProfile trait and qpudatashard-compatible envelope IDs.
//! - Multi-channel DeviceDiscovery layer (Bluetooth, RF/Zigbee, WiFi, SF-Comms).
//!
//! It is transport-agnostic and vendor-agnostic: concrete Bluetooth/Zigbee/etc.
//! stacks live in separate crates that implement the traits defined here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

//
// 1. BrainSpecs and HostBudget (20-W envelope)
//

/// Approximate brain-level power envelope derived from telemetry.
/// Values are expressed as fractions of a 20-W reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrainSpecs {
    /// Estimated daily brain energy budget in joules.
    pub daily_joules: f32,
    /// Safe joules per intense cognitive pass.
    pub safe_joules_per_pass: f32,
    /// Fraction of 20 W allocated to baseline operations.
    pub baseline_fraction: f32,
    /// Fraction for focused work.
    pub focus_fraction: f32,
    /// Fraction reserved for rehab/recovery.
    pub rehab_fraction: f32,
    /// Fraction reserved for sleep consolidation.
    pub sleep_fraction: f32,
    /// Thermal / systemic safety limits (normalized).
    pub thermal_limit: f32,
    pub vascular_limit: f32,
    pub last_updated: DateTime<Utc>,
}

/// HostBudget expresses how much of the 20-W envelope is currently
/// available for organic_cpu-adjacent kernels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostBudget {
    /// Fraction of 20 W currently usable for augmentation (0.0–1.0).
    pub usable_fraction: f32,
    /// Remaining joules for this period (e.g., per hour or per day).
    pub remaining_joules: f32,
    /// Mode-aware cap derived from BrainSpecs (baseline/focus/rehab/sleep).
    pub mode_cap_fraction: f32,
}

impl HostBudget {
    pub fn is_sufficient(&self, estimated_joules: f32, required_fraction: f32) -> bool {
        self.remaining_joules >= estimated_joules && self.usable_fraction >= required_fraction
    }
}

//
// 2. Multimodal telemetry → organic_cpu state
//

/// EEG band power snapshot (normalized).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EegBands {
    pub delta: f32,
    pub theta: f32,
    pub alpha: f32,
    pub beta: f32,
    pub gamma: f32,
}

/// Heart Rate Variability metrics (simplified).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HrvMetrics {
    pub rmssd_ms: f32,
    pub sdnn_ms: f32,
}

/// Core physiological signals relevant to organic_cpu state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioSignals {
    pub eeg: Option<EegBands>,
    pub hrv: Option<HrvMetrics>,
    pub il6_level: Option<f32>,       // inflammatory marker (arbitrary units)
    pub skin_temp_c: Option<f32>,
    pub eda_microsiemens: Option<f32>,
    pub respiration_rate_hz: Option<f32>,
    pub pain_level_0_10: Option<f32>,
}

/// High-level lifeforce / organicCPU state.
/// cy, zen, chi are normalized (0–1) conceptual axes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceState {
    pub cy: f32,          // cognitive throughput / activity
    pub zen: f32,         // calmness / stability
    pub chi: f32,         // systemic vitality / resilience
    pub cognitive_load: f32, // composite cognitive load (0–1)
}

/// Corridor channels: separate but co-regulated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorChannel {
    /// Instantaneous energy cost (normalized 0–1).
    pub energy_cost: f32,
    /// Long-term bio-karma accumulation (fatigue / wear).
    pub bio_karma: f32,
    /// Safety-stress (0–1, where 1 is near unsafe).
    pub safety_stress: f32,
}

/// Unified cognitive–motor corridors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorState {
    pub cognitive: CorridorChannel,
    pub motor: CorridorChannel,
    pub autonomic: CorridorChannel,
    pub visceral: CorridorChannel,
}

//
// 3–5. Lifeforce envelopes, QuantumRecedingEnvelope, and step-safety
//

/// LifeforceEnvelope encodes acceptable ranges for lifeforce axes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceEnvelope {
    pub cy_min: f32,
    pub cy_max: f32,
    pub zen_min: f32,
    pub zen_max: f32,
    pub chi_min: f32,
    pub chi_max: f32,
}

impl LifeforceEnvelope {
    pub fn contains(&self, state: &LifeforceState) -> bool {
        state.cy >= self.cy_min
            && state.cy <= self.cy_max
            && state.zen >= self.zen_min
            && state.zen <= self.zen_max
            && state.chi >= self.chi_min
            && state.chi <= self.chi_max
    }
}

/// QuantumRecedingEnvelope encodes per-step caps on multiple dimensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumRecedingEnvelope {
    pub max_energy_delta: f32,
    pub max_thermal_delta: f32,
    pub max_duty_cycle: f32,
    pub max_kernel_distance: f32,
    pub max_bio_impact: f32,
    /// Optional extended axes (e.g., legal complexity, social risk).
    pub extra_axes: Option<[f32; 2]>,
}

/// Footprint a kernel must expose before actuation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelFootprint {
    pub energy_delta: f32,
    pub thermal_delta: f32,
    pub duty_cycle: f32,
    pub kernel_distance: f32,
    pub bio_impact: f32,
    pub extra_axes: Option<[f32; 2]>,
}

impl QuantumRecedingEnvelope {
    pub fn step_is_safe(&self, fp: &KernelFootprint) -> bool {
        if fp.energy_delta > self.max_energy_delta {
            return false;
        }
        if fp.thermal_delta > self.max_thermal_delta {
            return false;
        }
        if fp.duty_cycle > self.max_duty_cycle {
            return false;
        }
        if fp.kernel_distance > self.max_kernel_distance {
            return false;
        }
        if fp.bio_impact > self.max_bio_impact {
            return false;
        }
        if let (Some(env_extra), Some(fp_extra)) = (self.extra_axes, fp.extra_axes) {
            for (e, f) in env_extra.iter().zip(fp_extra.iter()) {
                if f > e {
                    return false;
                }
            }
        }
        true
    }
}

//
// 6. Psych_risk and PDR (psych-density rate)
//

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PsychRiskState {
    Normal,
    Moderate,
    High,
}

/// PsychRisk aggregates physload, cogload, envrisk, devstress.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PsychRisk {
    pub physload: f32,
    pub cogload: f32,
    pub envrisk: f32,
    pub devstress: f32,
    pub state: PsychRiskState,
}

/// PDR (psych-density rate) caps how many risky evolution steps per loop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PsychDensityRate {
    pub max_risky_steps_per_loop: u32,
    pub risky_steps_used: u32,
}

impl PsychDensityRate {
    pub fn can_take_risky_step(&self) -> bool {
        self.risky_steps_used < self.max_risky_steps_per_loop
    }
}

//
// 7. Telemetrical-osteosis and sampling limits
//

/// TelemetricalOsteosis caps telemetry sampling and logging overhead.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetricalOsteosis {
    /// Maximum sampling duty cycle for organic_cpu monitors (0–1).
    pub max_sampling_duty: f32,
    /// Maximum telemetry bandwidth per interval (bytes/sec).
    pub max_bandwidth_bps: u32,
    /// Maximum logging volume per interval (bytes).
    pub max_log_bytes: u64,
}

//
// 8. OrganicCpuProfile and qpudatashards
//

/// Envelope IDs are carried by qpudatashards instead of raw envelopes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeIds {
    pub lifeforce_envelope_id: Uuid,
    pub qre_envelope_id: Uuid,
    pub psych_risk_policy_id: Uuid,
    pub biometric_sampling_policy_id: Uuid,
}

/// OrganicCpuSnapshot is the minimal state a kernel may see.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganicCpuSnapshot {
    pub timestamp: DateTime<Utc>,
    pub brain_specs: BrainSpecs,
    pub host_budget: HostBudget,
    pub lifeforce_state: LifeforceState,
    pub lifeforce_envelope: LifeforceEnvelope,
    pub corridors: CorridorState,
    pub psych_risk: PsychRisk,
    pub pdr: PsychDensityRate,
    pub telem_osteosis: TelemetricalOsteosis,
    pub qre: QuantumRecedingEnvelope,
    pub envelope_ids: EnvelopeIds,
}

/// Trait implemented by any host that exposes organic_cpu state and envelopes.
pub trait OrganicCpuProfile {
    /// Fetch a fresh snapshot for safety checks.
    fn snapshot(&self) -> OrganicCpuSnapshot;

    /// Update from telemetry; typically runs in a scheduler, not per call.
    fn update_from_telemetry(&mut self, bio: &BioSignals);

    /// Reserve energy from HostBudget for a planned kernel step.
    fn reserve_energy(&mut self, estimated_joules: f32, required_fraction: f32) -> bool;

    /// Register that a risky evolution step has been taken (for PDR).
    fn register_risky_step(&mut self);

    /// Access raw envelope IDs to embed into qpudatashards.
    fn envelope_ids(&self) -> EnvelopeIds;
}

//
// 9. Neurorights and augmented-citizen envelopes
//

/// Neurorights envelope for augmented citizens (simplified).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeurorightsEnvelope {
    pub max_cognitive_load: f32,
    pub max_duty_cycle: f32,
    pub max_legal_complexity: f32,
    pub forbid_affective_manipulation: bool,
    pub require_human_appeal_path: bool,
    pub jurisdiction_code: String,
}

//
// 10. Cybostate-factor and ResourceLedgers
//

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CybostateFactor {
    pub geo: f32,
    pub bio: f32,
    pub rights: f32,
}

impl CybostateFactor {
    pub fn value(&self) -> f32 {
        self.geo.min(self.bio).min(self.rights)
    }
}

/// Resource ledger for silicon resources, per agent or per host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceLedger {
    pub cpu_ms_budget: u64,
    pub cpu_ms_used: u64,
    pub gpu_ms_budget: u64,
    pub gpu_ms_used: u64,
    pub mem_bytes_budget: u64,
    pub mem_bytes_used: u64,
    pub net_bytes_budget: u64,
    pub net_bytes_used: u64,
}

impl ResourceLedger {
    pub fn can_allocate_cpu(&self, ms: u64) -> bool {
        self.cpu_ms_used + ms <= self.cpu_ms_budget
    }
}

//
// Device Discovery Layer (Bluetooth, RF/Zigbee, WiFi, SF-Comms)
//

/// High-level communications channel type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommChannelType {
    BluetoothLe,
    BluetoothClassic,
    RfMesh,      // Zigbee / 802.15.4 / custom RF mesh
    WifiLan,
    SfComms,     // software / soft-radio / virtual comms layer
    UsbSerial,
    Other(u8),   // reserved for future channels
}

/// Capability flags exposed by an organic_cpu-aware device.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub supports_lifeforce_query: bool,
    pub supports_organic_cpu_snapshot: bool,
    pub supports_envelope_negotiation: bool,
    pub supports_tinyml_inference: bool,
    pub supports_qpudatashard_routing: bool,
}

/// Basic identity for a device discovered over any channel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: Uuid,
    pub comm_type: CommChannelType,
    pub human_name: Option<String>,
    pub address: Option<String>, // e.g., MAC, IP, RF addr, virtual URI
    pub rssi_dbm: Option<i16>,
    pub capabilities: DeviceCapabilities,
}

/// Caller-facing filter for discovery operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryFilter {
    pub comm_types: Vec<CommChannelType>,
    pub require_organic_cpu_support: bool,
    pub min_rssi_dbm: Option<i16>,
}

/// Abstract adapter for a single communications channel (Bluetooth, RF, etc.).
pub trait CommAdapter {
    /// Underlying channel type.
    fn channel_type(&self) -> CommChannelType;

    /// Perform a discovery scan according to filter; non-blocking or bounded-time.
    fn discover(&mut self, filter: &DiscoveryFilter) -> Vec<DiscoveredDevice>;

    /// Open a lightweight control session with a discovered device.
    /// This should be minimal (no long-lived streams by default).
    fn open_session(&mut self, device: &DiscoveredDevice) -> Result<Box<dyn CommSession>, CommError>;
}

/// Session abstraction over a specific channel.
/// Implementors wrap BLE GATT, RF frames, WebSockets, etc.
pub trait CommSession {
    /// Send a small control message (e.g., “query organic snapshot”, “negotiate envelope”).
    fn send(&mut self, payload: &[u8]) -> Result<(), CommError>;

    /// Receive a control message (bounded in size).
    fn recv(&mut self) -> Result<Vec<u8>, CommError>;

    /// Close the session gracefully.
    fn close(&mut self) -> Result<(), CommError>;
}

/// Errors for discovery and comms.
#[derive(Debug, Error)]
pub enum CommError {
    #[error("channel not available")]
    ChannelUnavailable,
    #[error("discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("session failed: {0}")]
    SessionFailed(String),
}

/// DeviceDiscovery orchestrates multiple CommAdapters.
pub struct DeviceDiscovery<'a> {
    adapters: Vec<&'a mut dyn CommAdapter>,
}

impl<'a> DeviceDiscovery<'a> {
    pub fn new() -> Self {
        Self { adapters: Vec::new() }
    }

    pub fn register_adapter(&mut self, adapter: &'a mut dyn CommAdapter) {
        self.adapters.push(adapter);
    }

    /// Multi-channel discovery over all registered adapters.
    pub fn discover_devices(&mut self, filter: &DiscoveryFilter) -> Vec<DiscoveredDevice> {
        let mut out = Vec::new();
        for adapter in self.adapters.iter_mut() {
            if !filter.comm_types.is_empty()
                && !filter.comm_types.contains(&adapter.channel_type())
            {
                continue;
            }
            let mut devices = adapter.discover(filter);
            if filter.require_organic_cpu_support {
                devices.retain(|d| d.capabilities.supports_organic_cpu_snapshot);
            }
            if let Some(min_rssi) = filter.min_rssi_dbm {
                devices.retain(|d| d.rssi_dbm.map_or(false, |r| r >= min_rssi));
            }
            out.extend(devices);
        }
        out
    }
}

//
// Example: OrganicCpuProfile + discovery glue
//

/// High-level profile of a host plus comm/discovery hooks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganicCpuHostDescriptor {
    pub host_id: Uuid,
    pub profile_envelope_ids: EnvelopeIds,
    pub comm_channels: Vec<CommChannelType>,
    pub last_seen: DateTime<Utc>,
}

/// Abstract trait for anything that can expose an OrganicCpuProfile
/// over a comm session (phone, headset, edge box, embedded MCU).
pub trait RemoteOrganicCpuEndpoint {
    /// Build a control payload to query the snapshot.
    fn build_snapshot_request(&self) -> Vec<u8>;

    /// Parse a snapshot response into OrganicCpuSnapshot.
    fn parse_snapshot_response(&self, payload: &[u8]) -> Result<OrganicCpuSnapshot, CommError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyProfile {
        snapshot: OrganicCpuSnapshot,
    }

    impl OrganicCpuProfile for DummyProfile {
        fn snapshot(&self) -> OrganicCpuSnapshot {
            self.snapshot.clone()
        }

        fn update_from_telemetry(&mut self, _bio: &BioSignals) {
            // No-op for dummy; real impl updates BrainSpecs, LifeforceState, etc.
        }

        fn reserve_energy(&mut self, estimated_joules: f32, required_fraction: f32) -> bool {
            self.snapshot
                .host_budget
                .is_sufficient(estimated_joules, required_fraction)
        }

        fn register_risky_step(&mut self) {
            self.snapshot.pdr.risky_steps_used += 1;
        }

        fn envelope_ids(&self) -> EnvelopeIds {
            self.snapshot.envelope_ids.clone()
        }
    }

    #[test]
    fn quantum_receding_envelope_rejects_unsafe_step() {
        let env = QuantumRecedingEnvelope {
            max_energy_delta: 0.3,
            max_thermal_delta: 0.2,
            max_duty_cycle: 0.5,
            max_kernel_distance: 0.4,
            max_bio_impact: 0.4,
            extra_axes: None,
        };
        let fp_ok = KernelFootprint {
            energy_delta: 0.2,
            thermal_delta: 0.1,
            duty_cycle: 0.3,
            kernel_distance: 0.2,
            bio_impact: 0.3,
            extra_axes: None,
        };
        let fp_bad = KernelFootprint {
            energy_delta: 0.4,
            ..fp_ok.clone()
        };
        assert!(env.step_is_safe(&fp_ok));
        assert!(!env.step_is_safe(&fp_bad));
    }
}

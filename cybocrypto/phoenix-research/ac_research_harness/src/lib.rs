//! ac_research_harness
//!
//! A small, drop-in Rust crate to measure:
//! - End-to-end round-trip latency for Terminal → Companion → Wallet/Guard → Terminal.
//! - Corridor proxies (prompts/min, steps/task, hesitation times).
//! - On-device resource usage (CPU, memory).
//! - Policy decisions from Payment/Assistant engines.
//!
//! Designed for use across pumps, pin-pads, and kiosks in Phoenix-style pilots.
//!
//! This crate is intentionally storage-agnostic: it emits JSON records via
//! `tracing` so you can:
//! - Ship them to OpenTelemetry collectors / Datadog / Prometheus / log files.
//! - Or consume them directly in integration tests and offline analysis.
//!
//! Integration points:
//! - Call `SessionTelemetry::start(...)` when a terminal session begins.
//! - Wrap each Terminal → Companion → Wallet/Guard → Terminal loop in
//!   `SessionTelemetry::record_round_trip(...)`.
//! - Feed policy decisions into `record_policy_decision(...)`.
//! - Optionally, periodically sample device metrics via `sample_device_metrics()`.
//! - Call `init_telemetry_logging(...)` (and optionally `init_otel_pipeline(...)`)
//!   once at process startup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, System, SystemExt};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[cfg(feature = "otel")]
use opentelemetry::{
    global,
    sdk::{
        trace as sdktrace,
        Resource,
    },
    KeyValue,
};
#[cfg(feature = "otel")]
use opentelemetry_otlp::WithExportConfig;

/// Terminal types covered by the harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalType {
    FuelPump,
    PinPad,
    Kiosk,
    Other,
}

/// Interaction role / mode (aligned with your AugmentedCitizenProfile).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionMode {
    DriverAtPump,
    WalkUpAugmentedCitizen,
    WheelchairUser,
    LowVisionScreenReader,
    XRAugmented,
    NeuroAugmented,
    Unknown,
}

/// Payment category for the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentType {
    CivicBasic,
    RetailStandard,
    Regulated,
    EcoReward,
    HighValue,
    Unknown,
}

/// Reasons that triggered the loop (Consent / audit causes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentReason {
    RightsEnvelopeViolated,
    EssentialStateUnstable,
    PromptRateExceeded,
    PaymentRateExceeded,
    MaxAutoAmountExceeded,
    NetworkDegraded,
    TokenInvalid,
    EcoRewardEligible,
    AccessibilityAssistRequired,
    Other,
}

/// Where the assistant chose to surface a prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssistantChannel {
    ClerkTerminal,
    Kiosk,
    PersonalCompanion,
    SilentLog,
}

/// Policy decision snapshot (compatible with your policy engine).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentDecision {
    AllowAuto,
    RequireConfirmation,
    RequireHumanReview,
    Deny,
}

/// Latency and corridor metrics for a single round-trip cycle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundTripMetrics {
    /// Unique session id this round trip belongs to.
    pub session_id: Uuid,
    /// Monotonic epoch start for this round-trip (wall-clock approximated).
    pub started_at: DateTime<Utc>,
    /// Duration in milliseconds from terminal prompt to final terminal decision.
    pub duration_ms: u64,
    /// Whether the round-trip completed successfully.
    pub success: bool,
    /// Number of retries in this round-trip.
    pub retries: u32,
    /// Network profile label ("normal", "degraded", etc.).
    pub network_profile: String,
    /// Corridor proxies.
    pub prompts_count: u32,
    pub steps_count: u32,
    /// Sum of hesitation times in milliseconds across prompts in this round-trip.
    pub hesitation_ms_total: u64,
}

/// On-device resource telemetry snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceMetrics {
    pub sampled_at: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub mem_usage_percent: f32,
    /// Battery drain per hour estimate, if available, else None.
    pub battery_drain_per_hour: Option<f32>,
}

/// Combined policy decision telemetry for one loop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDecisionTelemetry {
    pub session_id: Uuid,
    pub round_trip_index: u32,
    pub payment_decision: PaymentDecision,
    pub assistant_primary_channel: AssistantChannel,
    pub assistant_secondary_channel: Option<AssistantChannel>,
    pub explanation_level: String,
    pub explanation_allowed: bool,
    pub log_silently: bool,
}

/// Session-level metadata for grouping metrics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub terminal_type: TerminalType,
    pub interaction_mode: InteractionMode,
    pub payment_type: PaymentType,
    pub is_basic_service: bool,
    pub network_profile: String,
}

/// Errors from the research harness.
#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("round-trip duration must be > 0 ms")]
    InvalidDuration,
    #[error("internal error: {0}")]
    Internal(String),
}

/// Central entry point for per-session telemetry.
pub struct SessionTelemetry {
    meta: SessionMetadata,
    round_trip_counter: u32,
}

impl SessionTelemetry {
    /// Begin a new session, generating a new session_id and emitting a start event.
    pub fn start(
        terminal_type: TerminalType,
        interaction_mode: InteractionMode,
        payment_type: PaymentType,
        is_basic_service: bool,
        network_profile: impl Into<String>,
    ) -> Self {
        let session_id = Uuid::new_v4();
        let started_at = Utc::now();
        let meta = SessionMetadata {
            session_id,
            started_at,
            terminal_type,
            interaction_mode,
            payment_type,
            is_basic_service,
            network_profile: network_profile.into(),
        };

        // Emit a structured event for session start.
        let json = serde_json::to_string(&meta).unwrap_or_else(|_| "{}".to_string());
        info!(
            target: "ac_research.session_start",
            session_id = %meta.session_id,
            %json
        );

        SessionTelemetry {
            meta,
            round_trip_counter: 0,
        }
    }

    /// Access the session id.
    pub fn session_id(&self) -> Uuid {
        self.meta.session_id
    }

    /// Record a single round-trip.
    ///
    /// Call this once you know the elapsed duration, success, and corridor proxy counts
    /// for a given Terminal → Companion → Wallet/Guard → Terminal cycle.
    pub fn record_round_trip(
        &mut self,
        duration_ms: u64,
        success: bool,
        retries: u32,
        prompts_count: u32,
        steps_count: u32,
        hesitation_ms_total: u64,
    ) -> Result<RoundTripMetrics, HarnessError> {
        if duration_ms == 0 {
            return Err(HarnessError::InvalidDuration);
        }

        self.round_trip_counter += 1;

        let metrics = RoundTripMetrics {
            session_id: self.meta.session_id,
            started_at: Utc::now(),
            duration_ms,
            success,
            retries,
            network_profile: self.meta.network_profile.clone(),
            prompts_count,
            steps_count,
            hesitation_ms_total,
        };

        let json = serde_json::to_string(&metrics)
            .unwrap_or_else(|_| "{}".to_string());
        info!(
            target: "ac_research.round_trip",
            session_id = %self.meta.session_id,
            round_trip_index = self.round_trip_counter,
            duration_ms = metrics.duration_ms,
            success = metrics.success,
            retries = metrics.retries,
            prompts = metrics.prompts_count,
            steps = metrics.steps_count,
            hesitation_ms_total = metrics.hesitation_ms_total,
            %json
        );

        Ok(metrics)
    }

    /// Record a policy decision linked to the latest round-trip.
    ///
    /// You typically call this right after running your PaymentPolicyEngine
    /// and AssistantPolicyEngine for the same cycle.
    pub fn record_policy_decision(
        &self,
        payment_decision: PaymentDecision,
        assistant_primary_channel: AssistantChannel,
        assistant_secondary_channel: Option<AssistantChannel>,
        explanation_level: impl Into<String>,
        explanation_allowed: bool,
        log_silently: bool,
    ) -> PolicyDecisionTelemetry {
        let telemetry = PolicyDecisionTelemetry {
            session_id: self.meta.session_id,
            round_trip_index: self.round_trip_counter,
            payment_decision,
            assistant_primary_channel,
            assistant_secondary_channel,
            explanation_level: explanation_level.into(),
            explanation_allowed,
            log_silently,
        };

        let json = serde_json::to_string(&telemetry)
            .unwrap_or_else(|_| "{}".to_string());

        info!(
            target: "ac_research.policy_decision",
            session_id = %self.meta.session_id,
            round_trip_index = self.round_trip_counter,
            payment_decision = ?telemetry.payment_decision,
            assistant_primary = ?telemetry.assistant_primary_channel,
            assistant_secondary = ?telemetry.assistant_secondary_channel,
            explanation_level = %telemetry.explanation_level,
            explanation_allowed = telemetry.explanation_allowed,
            log_silently = telemetry.log_silently,
            %json
        );

        telemetry
    }
}

/// Sample device-level resource metrics.
///
/// This is meant for edge boxes / companions (phone/headset / Pi-class devices),
/// not legacy POS hardware directly.
pub fn sample_device_metrics(sys: &mut System) -> DeviceMetrics {
    sys.refresh_memory();
    sys.refresh_cpu();

    let total_mem = sys.total_memory() as f32;
    let used_mem = sys.used_memory() as f32;
    let mem_usage_percent = if total_mem > 0.0 {
        (used_mem / total_mem) * 100.0
    } else {
        0.0
    };

    // Aggregate CPU usage across all CPUs.
    let mut cpu_total = 0.0_f32;
    let mut cpu_count = 0_u32;
    for cpu in sys.cpus() {
        cpu_total += cpu.cpu_usage();
        cpu_count += 1;
    }
    let cpu_usage_percent = if cpu_count > 0 {
        cpu_total / cpu_count as f32
    } else {
        0.0
    };

    // Battery metrics are not portable across all platforms; left as None.
    let metrics = DeviceMetrics {
        sampled_at: Utc::now(),
        cpu_usage_percent,
        mem_usage_percent,
        battery_drain_per_hour: None,
    };

    let json = serde_json::to_string(&metrics)
        .unwrap_or_else(|_| "{}".to_string());

    info!(
        target: "ac_research.device_metrics",
        cpu_usage_percent = metrics.cpu_usage_percent,
        mem_usage_percent = metrics.mem_usage_percent,
        %json
    );

    metrics
}

/// Initialize a simple tracing subscriber suitable for pilots.
///
/// This prints JSON lines to stdout and enables the targets used in this crate.
/// You can ship these logs through your existing pipeline, or combine with
/// OpenTelemetry when the `otel` feature is enabled.
pub fn init_telemetry_logging(default_level: &str) {
    // Example: default_level = "info" or "debug"
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "ac_research={level},ac_research_harness={level},info",
            level = default_level
        )
    });

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .flatten_event(true)
        .finish();

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        warn!("Failed to set global tracing subscriber: {}", e);
    }
}

/// Optional: initialize an OpenTelemetry pipeline for this process.
///
/// This follows the same general pattern as Datadog’s OTel guidance: you can
/// point `OTEL_EXPORTER_OTLP_ENDPOINT` at a Datadog Agent or any OTLP collector,
/// and use standard OTel env vars for service name and resource attributes.[attached_file:1]
#[cfg(feature = "otel")]
pub fn init_otel_pipeline(service_name: &str) {
    // Build a Resource describing this service.
    let mut resource = Resource::new(vec![KeyValue::new("service.name", service_name.to_string())]);

    // Additional attributes from environment (e.g., deployment.environment, service.version)
    // can be attached here if desired, similar to the Datadog example.
    if let Ok(env) = std::env::var("DEPLOYMENT_ENVIRONMENT") {
        resource = resource.merge(&Resource::new(vec![KeyValue::new(
            "deployment.environment",
            env,
        )]));
    }

    // Configure OTLP exporter over gRPC, using OTEL_EXPORTER_OTLP_ENDPOINT.
    let exporter = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            sdktrace::Config::default().with_resource(resource),
        )
        .install_simple()
        .expect("failed to initialize OTLP exporter");

    global::set_tracer_provider(exporter);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_start_and_round_trip_record() {
        init_telemetry_logging("debug");

        let mut sess = SessionTelemetry::start(
            TerminalType::FuelPump,
            InteractionMode::DriverAtPump,
            PaymentType::CivicBasic,
            true,
            "normal",
        );

        let rt = sess
            .record_round_trip(
                350,
                true,
                0,
                2, // prompts
                3, // steps
                800,
            )
            .expect("round-trip should be valid");

        assert_eq!(rt.duration_ms, 350);
        assert!(rt.success);
        assert_eq!(rt.prompts_count, 2);
        assert_eq!(rt.steps_count, 3);
    }

    #[test]
    fn policy_decision_record() {
        init_telemetry_logging("debug");

        let mut sess = SessionTelemetry::start(
            TerminalType::Kiosk,
            InteractionMode::LowVisionScreenReader,
            PaymentType::EcoReward,
            false,
            "degraded",
        );

        sess.record_round_trip(420, true, 1, 3, 4, 1200)
            .expect("round-trip ok");

        let decision = sess.record_policy_decision(
            PaymentDecision::AllowAuto,
            AssistantChannel::Kiosk,
            Some(AssistantChannel::PersonalCompanion),
            "Reasoned",
            true,
            false,
        );

        assert_eq!(decision.payment_decision, PaymentDecision::AllowAuto);
        assert_eq!(
            decision.assistant_primary_channel,
            AssistantChannel::Kiosk
        );
        assert_eq!(
            decision.assistant_secondary_channel,
            Some(AssistantChannel::PersonalCompanion)
        );
    }

    #[test]
    fn device_metrics_sample() {
        init_telemetry_logging("debug");

        let mut sys = System::new_all();
        let metrics = sample_device_metrics(&mut sys);
        // Sanity checks: percentages in [0, 100] range.
        assert!(metrics.cpu_usage_percent >= 0.0);
        assert!(metrics.cpu_usage_percent <= 100.0);
        assert!(metrics.mem_usage_percent >= 0.0);
        assert!(metrics.mem_usage_percent <= 100.0);
    }
}

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PsychBand {
    Low,
    Moderate,
    High,
}

#[derive(Clone, Copy, Debug)]
pub struct PsychRiskVector {
    pub physload: f32,   // 0.0–1.0
    pub cogload: f32,    // 0.0–1.0
    pub envrisk: f32,    // 0.0–1.0
    pub devstress: f32,  // 0.0–1.0
    pub scalar: f32,     // aggregated psych_risk 0.0–1.0
    pub band: PsychBand, // LOW / MODERATE / HIGH
}

#[derive(Clone, Copy, Debug)]
pub struct CorridorState {
    pub opacity: f32,        // 0.0–1.0 visual/cognitive opacity
    pub loop_hz: f32,        // sensory/control update frequency
    pub intensity_scale: f32 // global corridor intensity multiplier
}

#[derive(Clone, Copy, Debug)]
pub struct BloodBudget {
    pub current_tokens: f32, // remaining blood-linked tokens
    pub min_floor: f32,      // hard reserve, never drained
    pub max_per_min: f32,    // max allowed drain / minute
}

#[derive(Clone, Copy, Debug)]
pub struct WaveTokenBank {
    pub issued: f32,   // total WAVE tokens issued this session
    pub balance: f32,  // WAVE tokens currently available
    pub drain_rate: f32, // tokens/sec to drain while Dracula_Wave active
}

#[derive(Clone, Copy, Debug)]
pub struct DraculaWaveConfig {
    pub high_band_threshold: f32,     // typically 0.70
    pub corridor_opacity_max: f32,    // e.g. 0.95
    pub corridor_opacity_min: f32,    // e.g. 0.40
    pub loop_hz_min: f32,             // e.g. 0.50 Hz
    pub loop_hz_max: f32,             // e.g. 5.00 Hz
    pub intensity_min: f32,           // e.g. 0.40
    pub intensity_max: f32,           // e.g. 1.00
    pub rapid_delta: f32,             // e.g. 0.15 (FAST rise => critical)
    pub dwell_high_ms: u64,           // min time in HIGH before relaxing
    pub base_wave_issue: f32,         // tokens issued per activation
    pub max_wave_balance: f32,        // cap on outstanding WAVE tokens
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DraculaWaveMode {
    Idle,
    Active,
}

#[derive(Clone, Debug)]
pub struct DraculaWaveState {
    pub mode: DraculaWaveMode,
    pub last_risk: f32,
    pub entered_at: Option<Instant>,
    pub corridor: CorridorState,
    pub last_update: Instant,
    pub wave_bank: WaveTokenBank,
}

#[derive(Clone, Debug)]
pub struct DraculaWaveDecision {
    pub mode: DraculaWaveMode,
    pub corridor: CorridorState,
    pub blood_drain_allowed: bool,
    pub blood_drain_cap_per_min: f32,
    pub wave_tokens_spent: f32,
    pub wave_tokens_balance: f32,
    pub notes: &'static str,
}

/// Hex-stamp for Dracula_Wave host-module authorship (Bostrom / Googolswarm anchoring).
pub const DRACULA_WAVE_HOST_HEXSTAMP: &str =
    "0x7d2f9a3c5e1b8d4f6a0c9e2b7f3d1a5c8e4b6d0f2a9c7e1b3d5f7a9c1e3b5";

impl DraculaWaveState {
    pub fn new(now: Instant, cfg: DraculaWaveConfig) -> Self {
        let corridor = CorridorState {
            opacity: cfg.corridor_opacity_min,
            loop_hz: cfg.loop_hz_max,
            intensity_scale: cfg.intensity_min,
        };

        let wave_bank = WaveTokenBank {
            issued: 0.0,
            balance: 0.0,
            drain_rate: 0.0,
        };

        Self {
            mode: DraculaWaveMode::Idle,
            last_risk: 0.0,
            entered_at: None,
            corridor,
            last_update: now,
            wave_bank,
        }
    }
}

fn clamp_01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

fn compute_band(risk: f32) -> PsychBand {
    if risk >= 0.70 {
        PsychBand::High
    } else if risk >= 0.40 {
        PsychBand::Moderate
    } else {
        PsychBand::Low
    }
}

/// Core Dracula_Wave update:
/// - Detect critical-risk corridors (HIGH band or rapid spike).
/// - When Active: block further Blood-token drain and substitute corridor/WAVE load.
/// - WAVE tokens are issued on entry and drained smoothly over time, modeling
///   deferred "energy repayment" without harming the host.
pub fn update_dracula_wave(
    cfg: DraculaWaveConfig,
    state: &mut DraculaWaveState,
    psych: PsychRiskVector,
    blood: BloodBudget,
    now: Instant,
) -> DraculaWaveDecision {
    let dt = now
        .checked_duration_since(state.last_update)
        .unwrap_or_else(|| Duration::from_millis(0));
    state.last_update = now;

    let risk = clamp_01(psych.scalar);
    let delta = risk - state.last_risk;
    state.last_risk = risk;

    let band = compute_band(risk);
    let rapid = delta >= cfg.rapid_delta;

    let in_critical_corridor = matches!(band, PsychBand::High) || rapid;

    match state.mode {
        DraculaWaveMode::Idle => {
            // If host is in a critical corridor and cannot "pay" in blood,
            // activate Dracula_Wave: issue WAVE tokens and substitute psych_load.
            let can_pay_blood = (blood.current_tokens - blood.min_floor) > 0.0;

            if in_critical_corridor && !can_pay_blood {
                state.mode = DraculaWaveMode::Active;
                state.entered_at = Some(now);

                // Initialize corridor toward protective max opacity / low loop.
                state.corridor.opacity = cfg.corridor_opacity_max;
                state.corridor.loop_hz = cfg.loop_hz_min;
                state.corridor.intensity_scale = cfg.intensity_min;

                // Issue WAVE tokens for this activation, capped.
                let issue = cfg.base_wave_issue;
                let new_balance = (state.wave_bank.balance + issue).min(cfg.max_wave_balance);
                state.wave_bank.issued += new_balance - state.wave_bank.balance;
                state.wave_bank.balance = new_balance;

                // Drain rate: spend all issued tokens across dwell window.
                let dwell_secs = (cfg.dwell_high_ms as f32 / 1000.0).max(1.0);
                state.wave_bank.drain_rate = new_balance / dwell_secs;

                return DraculaWaveDecision {
                    mode: DraculaWaveMode::Active,
                    corridor: state.corridor,
                    blood_drain_allowed: false,
                    blood_drain_cap_per_min: 0.0,
                    wave_tokens_spent: 0.0,
                    wave_tokens_balance: state.wave_bank.balance,
                    notes: "Dracula_Wave activated: corridor/WAVE substitution, blood-drain halted",
                };
            }

            // Remain idle: normal blood governance applies.
            DraculaWaveDecision {
                mode: DraculaWaveMode::Idle,
                corridor: state.corridor,
                blood_drain_allowed: blood.current_tokens > blood.min_floor,
                blood_drain_cap_per_min: blood.max_per_min,
                wave_tokens_spent: 0.0,
                wave_tokens_balance: state.wave_bank.balance,
                notes: "Dracula_Wave idle: standard psych_risk and blood-token rules apply",
            }
        }
        DraculaWaveMode::Active => {
            // While Active, refuse further Blood-token drain: use WAVE tokens + corridor shaping.
            let elapsed_in_high = state
                .entered_at
                .and_then(|t| now.checked_duration_since(t))
                .unwrap_or_else(|| Duration::from_millis(0));

            let can_relax =
                !in_critical_corridor && (elapsed_in_high.as_millis() as u64) >= cfg.dwell_high_ms;

            // Drain WAVE tokens over dt (non-punitive, smooth).
            let dt_secs = dt.as_secs_f32();
            let mut spent = 0.0;
            if dt_secs > 0.0 && state.wave_bank.drain_rate > 0.0 && state.wave_bank.balance > 0.0 {
                let candidate = state.wave_bank.drain_rate * dt_secs;
                spent = candidate.min(state.wave_bank.balance);
                state.wave_bank.balance -= spent;
            }

            if can_relax {
                state.mode = DraculaWaveMode::Idle;
                state.entered_at = None;

                // Smoothly bring corridor back toward normal values.
                state.corridor.opacity = cfg.corridor_opacity_min;
                state.corridor.loop_hz = cfg.loop_hz_max;
                state.corridor.intensity_scale = cfg.intensity_max;

                return DraculaWaveDecision {
                    mode: DraculaWaveMode::Idle,
                    corridor: state.corridor,
                    blood_drain_allowed: blood.current_tokens > blood.min_floor,
                    blood_drain_cap_per_min: blood.max_per_min,
                    wave_tokens_spent: spent,
                    wave_tokens_balance: state.wave_bank.balance,
                    notes: "Dracula_Wave de-escalated: corridor normalized, blood rules restored",
                };
            }

            // Stay Active: shape corridor based on current psych_risk.
            // Higher risk => more opacity, lower loop frequency.
            let denom = (1.0 - cfg.high_band_threshold).max(0.001);
            let risk_scale = clamp_01((risk - cfg.high_band_threshold) / denom);

            let target_opacity = cfg.corridor_opacity_min
                + (cfg.corridor_opacity_max - cfg.corridor_opacity_min) * risk_scale;

            let target_loop_hz =
                cfg.loop_hz_max - (cfg.loop_hz_max - cfg.loop_hz_min) * risk_scale;

            // Intensity kept in a narrow band (protective, not punishing).
            let target_intensity = cfg.intensity_min
                + (cfg.intensity_max - cfg.intensity_min) * (1.0 - risk_scale);

            // Simple first-order smoothing over dt.
            let ms = dt.as_millis() as f32;
            let alpha = if ms <= 0.0 {
                0.0
            } else {
                (ms / 300.0).min(1.0)
            };

            state.corridor.opacity += alpha * (target_opacity - state.corridor.opacity);
            state.corridor.loop_hz += alpha * (target_loop_hz - state.corridor.loop_hz);
            state.corridor.intensity_scale +=
                alpha * (target_intensity - state.corridor.intensity_scale);

            DraculaWaveDecision {
                mode: DraculaWaveMode::Active,
                corridor: state.corridor,
                blood_drain_allowed: false, // core protection: no extra blood drain
                blood_drain_cap_per_min: 0.0,
                wave_tokens_spent: spent,
                wave_tokens_balance: state.wave_bank.balance,
                notes: "Dracula_Wave active: corridor & WAVE tokens managing psych_load safely",
            }
        }
    }
}

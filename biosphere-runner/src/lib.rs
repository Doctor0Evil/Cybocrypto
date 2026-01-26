use std::time::SystemTime;

// --- Core host snapshot types (aligned with OrganicCpuProfile) ---

#[derive(Clone, Copy, Debug)]
pub enum CorridorLoadBand {
    Low,
    Moderate,
    High,
}

#[derive(Clone, Copy, Debug)]
pub enum PsychRisk {
    Normal,
    Moderate,
    High,
}

#[derive(Clone, Copy, Debug)]
pub struct LifeforceState {
    pub cy: f32,   // cognitive clarity
    pub zen: f32,  // calm / affect balance
    pub chi: f32,  // perceived energy / resilience
}

#[derive(Clone, Copy, Debug)]
pub struct CorridorState {
    pub cognitive: CorridorLoadBand,
    pub motor: CorridorLoadBand,
    pub autonomic: CorridorLoadBand,
    pub visceral: CorridorLoadBand,
}

#[derive(Clone, Copy, Debug)]
pub struct OrganicCpuSnapshot {
    pub captured_at: SystemTime,
    pub lifeforce: LifeforceState,
    pub corridors: CorridorState,
    pub psych_risk: PsychRisk,
    pub eco_flops_epoch: u64,
    pub eco_energy_nj_epoch: f32,
}

// --- Kernel footprint for biosphere actions (5D safety vector) ---

#[derive(Clone, Copy, Debug)]
pub struct KernelFootprint {
    pub energy_delta_nj: f32,
    pub duty_cycle_pct: f32,
    pub duration_ms: u32,
    pub motor_complexity: f32,      // 0–1
    pub cognitive_complexity: f32,  // 0–1
}

// --- QuantumRecedingEnvelope: per-step safety limits ---

#[derive(Clone, Copy, Debug)]
pub struct QuantumRecedingEnvelope {
    pub max_energy_delta_nj: f32,
    pub max_duty_cycle_pct: f32,
    pub max_duration_ms: u32,
    pub max_motor_complexity: f32,
    pub max_cognitive_complexity: f32,
}

impl QuantumRecedingEnvelope {
    pub fn step_is_safe(
        &self,
        host: &OrganicCpuSnapshot,
        footprint: &KernelFootprint,
    ) -> bool {
        if footprint.energy_delta_nj > self.max_energy_delta_nj {
            return false;
        }
        if footprint.duty_cycle_pct > self.max_duty_cycle_pct {
            return false;
        }
        if footprint.duration_ms > self.max_duration_ms {
            return false;
        }
        if footprint.motor_complexity > self.max_motor_complexity {
            return false;
        }
        if footprint.cognitive_complexity > self.max_cognitive_complexity {
            return false;
        }

        // tighten limits under elevated psych risk or overloaded corridors
        match host.psych_risk {
            PsychRisk::High => {
                if footprint.cognitive_complexity > 0.4 {
                    return false;
                }
                if footprint.motor_complexity > 0.4 {
                    return false;
                }
            }
            PsychRisk::Moderate => {
                if footprint.cognitive_complexity > 0.7 {
                    return false;
                }
            }
            PsychRisk::Normal => {}
        }

        if let CorridorLoadBand::High = host.corridors.motor {
            if footprint.motor_complexity > 0.5 {
                return false;
            }
        }
        if let CorridorLoadBand::High = host.corridors.cognitive {
            if footprint.cognitive_complexity > 0.5 {
                return false;
            }
        }

        true
    }
}

// --- Biosphere capability and AI-chat runner interface ---

#[derive(Clone, Copy, Debug)]
pub enum BiosphereCapabilityKind {
    SelectMenu,
    ReachObject,
    RotateObject,
    Teleport,
    PaymentConfirm,
}

#[derive(Clone, Copy, Debug)]
pub struct BiosphereCapabilityRequest {
    pub kind: BiosphereCapabilityKind,
    pub context_tag: &'static str, // e.g. "civic", "gaming", "rehab"
}

#[derive(Clone, Copy, Debug)]
pub struct NeurorightsEnvelope {
    pub allow_affective_influence: bool,
    pub max_cognitive_load: f32,
    pub jurisdiction_code: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct CybostateFactor {
    pub c_geo_bio_rights: f32, // 0–1 global veto scalar
}

#[derive(Clone, Copy, Debug)]
pub struct EcoBudget {
    pub brain_tokens_available: f32,
    pub eco_tokens_available: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RunnerDecision {
    pub allowed: bool,
    pub reason: &'static str,
}

pub trait BiosphereRunner {
    fn decide_biosphere_step(
        &self,
        host: &OrganicCpuSnapshot,
        envelope: &QuantumRecedingEnvelope,
        neurorights: &NeurorightsEnvelope,
        cybostate: &CybostateFactor,
        eco: &EcoBudget,
        request: &BiosphereCapabilityRequest,
        footprint: &KernelFootprint,
    ) -> RunnerDecision;
}

// --- Default implementation for AI-chat + dexterity biosphere ---

pub struct DefaultBiosphereRunner;

impl DefaultBiosphereRunner {
    fn base_footprint_for(request: &BiosphereCapabilityRequest) -> KernelFootprint {
        match request.kind {
            BiosphereCapabilityKind::SelectMenu => KernelFootprint {
                energy_delta_nj: 50.0,
                duty_cycle_pct: 2.0,
                duration_ms: 500,
                motor_complexity: 0.2,
                cognitive_complexity: 0.3,
            },
            BiosphereCapabilityKind::ReachObject => KernelFootprint {
                energy_delta_nj: 80.0,
                duty_cycle_pct: 4.0,
                duration_ms: 800,
                motor_complexity: 0.5,
                cognitive_complexity: 0.4,
            },
            BiosphereCapabilityKind::RotateObject => KernelFootprint {
                energy_delta_nj: 60.0,
                duty_cycle_pct: 3.0,
                duration_ms: 700,
                motor_complexity: 0.4,
                cognitive_complexity: 0.4,
            },
            BiosphereCapabilityKind::Teleport => KernelFootprint {
                energy_delta_nj: 120.0,
                duty_cycle_pct: 5.0,
                duration_ms: 900,
                motor_complexity: 0.6,
                cognitive_complexity: 0.6,
            },
            BiosphereCapabilityKind::PaymentConfirm => KernelFootprint {
                energy_delta_nj: 70.0,
                duty_cycle_pct: 3.0,
                duration_ms: 600,
                motor_complexity: 0.2,
                cognitive_complexity: 0.5,
            },
        }
    }
}

impl BiosphereRunner for DefaultBiosphereRunner {
    fn decide_biosphere_step(
        &self,
        host: &OrganicCpuSnapshot,
        envelope: &QuantumRecedingEnvelope,
        neurorights: &NeurorightsEnvelope,
        cybostate: &CybostateFactor,
        eco: &EcoBudget,
        request: &BiosphereCapabilityRequest,
        footprint_override: &KernelFootprint,
    ) -> RunnerDecision {
        if cybostate.c_geo_bio_rights < 0.7 {
            return RunnerDecision {
                allowed: false,
                reason: "Global CybostateFactor veto",
            };
        }

        if eco.brain_tokens_available <= 0.0 || eco.eco_tokens_available <= 0.0 {
            return RunnerDecision {
                allowed: false,
                reason: "Insufficient brain/eco token budget",
            };
        }

        if !neurorights.allow_affective_influence
            && matches!(request.kind, BiosphereCapabilityKind::PaymentConfirm)
            && request.context_tag == "civic"
        {
            return RunnerDecision {
                allowed: false,
                reason: "NeurorightsEnvelope blocks affective payment prompt",
            };
        }

        let mut fp = Self::base_footprint_for(request);
        fp.energy_delta_nj = fp.energy_delta_nj.max(footprint_override.energy_delta_nj);
        fp.duty_cycle_pct = fp.duty_cycle_pct.max(footprint_override.duty_cycle_pct);
        fp.duration_ms = fp.duration_ms.max(footprint_override.duration_ms);
        fp.motor_complexity = fp.motor_complexity.max(footprint_override.motor_complexity);
        fp.cognitive_complexity = fp
            .cognitive_complexity
            .max(footprint_override.cognitive_complexity);

        if fp.cognitive_complexity > neurorights.max_cognitive_load {
            return RunnerDecision {
                allowed: false,
                reason: "Exceeds neurorights cognitive load",
            };
        }

        if !envelope.step_is_safe(host, &fp) {
            return RunnerDecision {
                allowed: false,
                reason: "QuantumRecedingEnvelope step_is_safe=false",
            };
        }

        RunnerDecision {
            allowed: true,
            reason: "Allowed by envelopes and budgets",
        }
    }
}

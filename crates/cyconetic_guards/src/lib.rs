use cyconetic_ledger::{KerVector, NeurorightsEnvelopeRef, NeurorightsBoundPromptEnvelope};

pub const ROH_HARD_CEILING: f32 = 0.3;
pub const KF_DEFAULT: f32 = 0.93;
pub const ROH_DEFAULT: f32 = 0.08;
pub const CSF_DEFAULT: f32 = 0.87;

// ALN-compiled guards (called from generated code or hand-written routers)

#[inline]
pub fn requires_roh_below(ker: &KerVector, max_roh: f32) -> Result<(), String> {
    if ker.risk_of_harm <= max_roh {
        Ok(())
    } else {
        Err(format!(
            "RoH {} above ceiling {}",
            ker.risk_of_harm, max_roh
        ))
    }
}

#[inline]
pub fn requires_cybostate_min(ker: &KerVector, min_csf: f32) -> Result<(), String> {
    if ker.cybostate_factor >= min_csf {
        Ok(())
    } else {
        Err(format!(
            "CSF {} below minimum {}",
            ker.cybostate_factor, min_csf
        ))
    }
}

#[inline]
pub fn requires_neurorights_envelope(
    env: &NeurorightsEnvelopeRef,
    expected: &str,
) -> Result<(), String> {
    if env.0 == expected {
        Ok(())
    } else {
        Err(format!(
            "Neurorights envelope mismatch: got {}, expected {}",
            env.0, expected
        ))
    }
}

// Convenience guard that applies a typical neurorights profile.
pub fn enforce_standard_ker_and_neurorights(
    env: &NeurorightsBoundPromptEnvelope,
    domain_min_csf: f32,
    expected_envelope: &str,
) -> Result<(), String> {
    requires_roh_below(&env.ker, ROH_HARD_CEILING)?;
    requires_cybostate_min(&env.ker, domain_min_csf)?;
    requires_neurorights_envelope(&env.neurorights_profile, expected_envelope)?;
    Ok(())
}

// Macro surface that ALN shards can compile to.
#[macro_export]
macro_rules! requires_roh_below {
    ($ker:expr, $max:expr) => {
        $crate::requires_roh_below($ker, $max)?
    };
}

#[macro_export]
macro_rules! requires_cybostate {
    ($ker:expr, min = $min:expr) => {
        $crate::requires_cybostate_min($ker, $min)?
    };
}

#[macro_export]
macro_rules! requires_neurorights_envelope {
    ($env:expr, $id:expr) => {
        $crate::requires_neurorights_envelope($env, $id)?
    };
}

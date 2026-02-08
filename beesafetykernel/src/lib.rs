#![no_std]

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum CorridorKind {
    Thermal,
    Acoustic,
    EMF,
    Chemical,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CorridorEnvelope {
    pub kind: CorridorKind,
    pub l_min: f32,
    pub l_max: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BeeContext {
    pub sensitivity: f32,       // >= 1.0
    pub in_hive_exclusion: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub node_id: u32,
    pub duty_cycle: f32,        // [0,1]
    pub bee_ctx: BeeContext,
    pub predicted_levels: [f32; 4], // EMF, Thermal, Acoustic, Chemical
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct KernelParams {
    pub phi_ref: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct KernelDecision {
    pub safe_duty_cycle: f32,
    pub permitted: bool,
    pub phi_penalty: f32,
}

#[derive(Debug)]
pub enum KernelError {
    EmptyCorridors,
    InvalidDutyCycle,
    MismatchedModalities,
}

pub struct BeeSafetyKernel {
    envelopes: [CorridorEnvelope; 4],
    params: KernelParams,
}

impl BeeSafetyKernel {
    pub fn new(
        envelopes: [CorridorEnvelope; 4],
        params: KernelParams,
    ) -> Result<Self, KernelError> {
        Ok(Self { envelopes, params })
    }

    fn compute_phi(
        &self,
        bee_ctx: &BeeContext,
        levels: &[f32; 4],
    ) -> f32 {
        let mut phi = 0.0f32;
        for (i, env) in self.envelopes.iter().enumerate() {
            let l = levels[i];
            let d_high = (l - env.l_max).max(0.0);
            let d_low  = (env.l_min - l).max(0.0);
            let d = if d_high > 0.0 { d_high } else { d_low };
            if d > 0.0 {
                phi += d * d;
            }
        }
        if bee_ctx.in_hive_exclusion && phi > 0.0 {
            phi *= 1.0e6;
        }
        phi
    }

    pub fn evaluate_node(
        &self,
        state: &NodeState,
    ) -> Result<KernelDecision, KernelError> {
        if !(0.0..=1.0).contains(&state.duty_cycle) {
            return Err(KernelError::InvalidDutyCycle);
        }
        let phi = self.compute_phi(&state.bee_ctx, &state.predicted_levels);
        let permitted = (phi == 0.0) && !state.bee_ctx.in_hive_exclusion;
        let safe_duty = if permitted { state.duty_cycle } else { 0.0 };
        Ok(KernelDecision {
            safe_duty_cycle: safe_duty,
            permitted,
            phi_penalty: phi / self.params.phi_ref.max(1e-6),
        })
    }
}

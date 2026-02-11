use crate::cyberswarm::bees::{BeeCorridor, BeeSafetyKernel};
use crate::nature_bounds::{NatureBounds, MetabolicConstraint};
use crate::evolution::EvolutionProposalRecord;

/// MarineSafetyKernel: Species-agnostic, but marine-first. Harder than human kernel.
/// Uses same math as BeeSafetyKernel, but calibrated to ocean variables.
#[derive(Debug, Clone)]
pub struct MarineSafetyKernel {
    pub corridors: Vec<MarineCorridor>,
    pub residual_weights: HashMap<String, f64>,
    pub sovereignty_scalar: f64, // HBrating for marine: 0.0 = collapse, 1.0 = thriving
    pub last_calibration: u64,
}

#[derive(Debug, Clone)]
pub struct MarineCorridor {
    pub name: String, // e.g., "coral_spawn_2026", "migratory_path_north"
    pub safe: (f64, f64),
    pub warning: (f64, f64),
    pub hard_wall: (f64, f64),
    pub stressors: Vec<String>, // e.g., ["noise", "hypoxia", "microplastic", "sonar", "nanowave"]
}

impl MarineSafetyKernel {
    pub fn new() -> Self {
        Self {
            corridors: vec![
                MarineCorridor {
                    name: "coral_spawn_2026".to_string(),
                    safe: (26.5, 29.0),
                    warning: (25.0, 30.5),
                    hard_wall: (23.0, 32.0),
                    stressors: vec![
                        "noise".to_string(),
                        "hypoxia".to_string(),
                        "microplastic".to_string(),
                        "sonar".to_string(),
                        "nanowave".to_string(),
                    ],
                },
                MarineCorridor {
                    name: "migratory_path_north".to_string(),
                    safe: (18.0, 22.0),
                    warning: (15.0, 25.0),
                    hard_wall: (12.0, 28.0),
                    stressors: vec![
                        "noise".to_string(),
                        "chemical_runoff".to_string(),
                        "sonar".to_string(),
                        "nanowave".to_string(),
                    ],
                },
            ],
            residual_weights: HashMap::from([
                ("sonar".to_string(), 1.0),
                ("nanowave".to_string(), 0.98),
                ("hypoxia".to_string(), 1.0),
                ("microplastic".to_string(), 0.95),
                ("chemical_runoff".to_string(), 0.97),
                ("noise".to_string(), 0.85),
                ("temperature".to_string(), 0.7),
            ]),
            sovereignty_scalar: 1.0,
            last_calibration: 1708896000,
        }
    }

    pub fn calculate_residual(&self, observations: &HashMap<String, f64>) -> f64 {
        let mut v = 0.0;
        for (stressor, weight) in &self.residual_weights {
            if let Some(observed) = observations.get(stressor) {
                let normalized = self.normalize_stressor(stressor, *observed);
                v += weight * normalized * normalized;
            }
        }
        v
    }

    fn normalize_stressor(&self, stressor: &str, value: f64) -> f64 {
        for corridor in &self.corridors {
            if corridor.stressors.contains(&stressor.to_string()) {
                let (low, high) = corridor.hard_wall;
                if value < low || value > high {
                    return 1.0;
                }
                if value < corridor.warning.0 || value > corridor.warning.1 {
                    return 0.7;
                }
                if value >= corridor.safe.0 && value <= corridor.safe.1 {
                    return 0.0;
                }
                let range = corridor.warning.1 - corridor.warning.0;
                let pos = if value < corridor.warning.0 {
                    (value - corridor.warning.0) / range
                } else {
                    (value - corridor.warning.1) / range
                };
                return pos.abs() * 0.7;
            }
        }
        0.0
    }

    pub fn may_act(&self, current_v: f64, new_v: f64, sovereignty_scalar: f64) -> bool {
        if sovereignty_scalar < 0.2 {
            return false;
        }
        if new_v > current_v {
            return false;
        }
        true
    }

    pub fn allow_proposal(&self, proposal: &EvolutionProposalRecord) -> bool {
        let v_current = self.calculate_residual(&proposal.observations);
        let v_new = self.calculate_residual(&proposal.observations_new);
        self.may_act(v_current, v_new, self.sovereignty_scalar)
    }
}

/// MarineConsentFrame: For research, monitoring, simulation — never actuation.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarineConsentFrame {
    pub simulation_id: String,
    pub description: String,
    pub proposed_corridors: Vec<MarineCorridor>,
    pub predicted_v_marine: f64,
    pub consented_by: Vec<String>,
    pub simulation_only: bool,
    pub timestamp: u64,
    pub hexstamp: String, // GPG brainpoolP256r1/B088B85F5F631492
}

impl MarineConsentFrame {
    pub fn new_simulation_only(description: &str, corridors: Vec<MarineCorridor>) -> Self {
        Self {
            simulation_id: format!("marine_sim_{}", chrono::Utc::now().timestamp_millis()),
            description: description.to_string(),
            proposed_corridors: corridors,
            predicted_v_marine: 0.0,
            consented_by: vec![],
            simulation_only: true,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            hexstamp: "GPG:brainpoolP256r1/B088B85F5F631492".to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.simulation_only && self.hexstamp.starts_with("GPG:brainpoolP256r1/")
    }
}
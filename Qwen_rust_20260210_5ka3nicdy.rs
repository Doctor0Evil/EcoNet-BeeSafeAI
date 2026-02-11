use crate::nature_bounds::{NatureBounds, MetabolicConstraint, ViabilityKernel};
use crate::aln::AlnParticle;
use crate::sovereigntycore::invariants::RoHAxis;

/// BeeSafetyKernel: The non-negotiable envelope for honeybees. Harder than human kernel.
/// This is the root of the Bee Tree-of-Life. No human proposal may weaken it.
#[derive(Debug, Clone)]
pub struct BeeSafetyKernel {
    pub corridors: Vec<BeeCorridor>,
    pub residual_weights: HashMap<String, f64>, // e.g., "EMF" → 0.9, "pesticide" → 1.0
    pub sovereignty_scalar: f64, // BeeHBScore: 0.0 = collapse, 1.0 = thriving
    pub last_calibration: u64,   // Unix timestamp of last TDICSI/ESN calibration
}

#[derive(Debug, Clone)]
pub struct BeeCorridor {
    pub name: String, // e.g., "foraging_path_7", "brood_chamber"
    pub safe: (f64, f64), // e.g., temp: 32.0–35.0°C
    pub warning: (f64, f64), // e.g., temp: 30.0–37.0°C
    pub hard_wall: (f64, f64), // e.g., temp: 28.0–39.0°C
    pub stressors: Vec<String>, // e.g., ["EMF", "noise", "chemicals"]
}

impl BeeSafetyKernel {
    pub fn new() -> Self {
        Self {
            corridors: vec![
                BeeCorridor {
                    name: "brood_chamber".to_string(),
                    safe: (34.0, 35.0),
                    warning: (32.0, 37.0),
                    hard_wall: (30.0, 39.0),
                    stressors: vec!["EMF".to_string(), "noise".to_string(), "pesticide".to_string()],
                },
                BeeCorridor {
                    name: "foraging_path_7".to_string(),
                    safe: (28.0, 32.0),
                    warning: (25.0, 35.0),
                    hard_wall: (22.0, 38.0),
                    stressors: vec!["EMF".to_string(), "chemicals".to_string(), "nanowave".to_string()],
                },
            ],
            residual_weights: HashMap::from([
                ("EMF".to_string(), 0.95),
                ("pesticide".to_string(), 1.0),
                ("nanowave".to_string(), 0.98),
                ("noise".to_string(), 0.8),
                ("temperature".to_string(), 0.7),
            ]),
            sovereignty_scalar: 1.0,
            last_calibration: 1708896000, // 2024-01-25T00:00:00Z
        }
    }

    /// Calculate Lyapunov residual V_bee for current state
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
                    return 1.0; // Outside hard wall = max risk
                }
                if value < corridor.warning.0 || value > corridor.warning.1 {
                    return 0.7; // In warning zone
                }
                if value >= corridor.safe.0 && value <= corridor.safe.1 {
                    return 0.0; // In safe zone
                }
                // Linear interpolation in warning zone
                let range = corridor.warning.1 - corridor.warning.0;
                let pos = if value < corridor.warning.0 {
                    (value - corridor.warning.0) / range
                } else {
                    (value - corridor.warning.1) / range
                };
                return pos.abs() * 0.7;
            }
        }
        0.0 // Unknown stressor = no impact
    }

    /// Enforce: "No act if V_bee increases or sovereignty_scalar drops below 0.2"
    /// This is the core gate for all Cyberswarm actuators.
    pub fn may_act(&self, current_v: f64, new_v: f64, sovereignty_scalar: f64) -> bool {
        if sovereignty_scalar < 0.2 {
            return false; // Hive is failing — halt all activity
        }
        if new_v > current_v {
            return false; // V_bee must not increase
        }
        true
    }

    /// Returns true if a proposal (e.g., drone deployment, RF pulse) is allowed to execute
    pub fn allow_proposal(&self, proposal: &EvolutionProposalRecord) -> bool {
        let v_current = self.calculate_residual(&proposal.observations);
        let v_new = self.calculate_residual(&proposal.observations_new);
        self.may_act(v_current, v_new, self.sovereignty_scalar)
    }
}

/// BeeMutationConsentFrame: A particle that MUST be signed and stored, but CANNOT trigger real-world actuation.
#[derive(Debug, Serialize, Deserialize)]
pub struct BeeMutationConsentFrame {
    pub simulation_id: String,
    pub description: String,
    pub proposed_corridors: Vec<BeeCorridor>,
    pub predicted_v_bee: f64,
    pub consented_by: Vec<String>, // e.g., [bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7]
    pub simulation_only: bool,     // MUST be true. If false, reject at kernel level.
    pub timestamp: u64,
    pub hexstamp: String,          // SHA3-256 forbidden. Use GPG brainpoolP256r1 signature
}

impl BeeMutationConsentFrame {
    pub fn new_simulation_only(description: &str, corridors: Vec<BeeCorridor>) -> Self {
        Self {
            simulation_id: format!("sim_{}", chrono::Utc::now().timestamp_millis()),
            description: description.to_string(),
            proposed_corridors: corridors,
            predicted_v_bee: 0.0,
            consented_by: vec![],
            simulation_only: true, // HARD ENFORCED — this field is immutable in .aln schema
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            hexstamp: "GPG:brainpoolP256r1/B088B85F5F631492".to_string(), // Your key
        }
    }

    pub fn is_valid(&self) -> bool {
        self.simulation_only && self.hexstamp.starts_with("GPG:brainpoolP256r1/")
    }
}
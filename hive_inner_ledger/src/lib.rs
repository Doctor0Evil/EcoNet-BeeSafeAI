//! HiveInnerLedger crate: Sovereign ledger for bee corridor governance in environmental cybernetics.
//! Enforces RoH <= 0.3, passive telemetry, and human-eco proxy adjustments without hive actuation.
//! Integrates ALN for adaptive learning in Bostrom/DID-anchored environments.

use std::collections::HashMap;
use std::fmt;

/// Represents Risk-of-Harm (RoH) with strict threshold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoH(f64);

impl RoH {
    /// Creates a new RoH, clamping to valid range [0.0, 1.0].
    pub fn new(value: f64) -> Self {
        RoH(value.clamp(0.0, 1.0))
    }

    /// Checks if RoH is within safe limit (<= 0.3).
    pub fn is_safe(&self) -> bool {
        self.0 <= 0.3
    }
}

/// Enum for eco-corridor adjustments, constrained to human/infrastructure only.
#[derive(Clone, Debug, PartialEq)]
pub enum Adjustment {
    PlantingSchedule(String),  // e.g., "Spring cycle optimization"
    IrrigationLevel(f64),      // Water flow in liters per hectare
    HabitatConnectivity(Vec<(f64, f64)>),  // Coordinate pairs for connectivity paths
}

/// Trait for sovereign ledger operations, ensuring bee-centered invariants.
pub trait SovereignLedger {
    /// Applies an adjustment if it maintains RoH safety and no corridor violation.
    fn apply_adjustment(&mut self, adj: Adjustment, proposed_roh: RoH) -> Result<(), LedgerError>;

    /// Retrieves current RoH.
    fn current_roh(&self) -> RoH;

    /// Passive telemetry query, corridor-bound only.
    fn query_telemetry(&self, corridor_id: &str) -> Option<TelemetryData>;
}

/// Error types for ledger operations.
#[derive(Debug, PartialEq)]
pub enum LedgerError {
    RoHExceeded,
    CorridorViolation,
    StressDriftDetected,
}

/// Struct for bee telemetry data, passive and cross-modal.
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryData {
    pub brood_viability: f64,     // 0.0 to 1.0
    pub thermoregulation: f64,    // Temperature stability metric
    pub forage_stability: f64,    // Resource availability index
}

/// Implementation of HiveInnerLedger.
#[derive(Debug)]
pub struct HiveInnerLedger {
    roh: RoH,
    adjustments: Vec<Adjustment>,
    telemetry: HashMap<String, TelemetryData>,  // Keyed by corridor_id
    invariants: BeeSafetyInvariants,
}

/// Bee safety invariants, embedded for neurorights constraints.
#[derive(Debug)]
pub struct BeeSafetyInvariants {
    max_roh: RoH,
    no_chronic_stress: bool,
    no_violation: bool,
}

impl HiveInnerLedger {
    /// Initializes a new ledger with default safe state.
    pub fn new() -> Self {
        HiveInnerLedger {
            roh: RoH::new(0.0),
            adjustments: Vec::new(),
            telemetry: HashMap::new(),
            invariants: BeeSafetyInvariants {
                max_roh: RoH::new(0.3),
                no_chronic_stress: true,
                no_violation: true,
            },
        }
    }

    /// Adds telemetry data for a corridor, passively.
    pub fn add_telemetry(&mut self, corridor_id: String, data: TelemetryData) {
        self.telemetry.insert(corridor_id, data);
    }
}

impl SovereignLedger for HiveInnerLedger {
    fn apply_adjustment(&mut self, adj: Adjustment, proposed_roh: RoH) -> Result<(), LedgerError> {
        if !proposed_roh.is_safe() || proposed_roh > self.invariants.max_roh {
            return Err(LedgerError::RoHExceeded);
        }
        if !self.invariants.no_violation {
            return Err(LedgerError::CorridorViolation);
        }
        if !self.invariants.no_chronic_stress {
            return Err(LedgerError::StressDriftDetected);
        }
        self.adjustments.push(adj);
        self.roh = proposed_roh;
        Ok(())
    }

    fn current_roh(&self) -> RoH {
        self.roh
    }

    fn query_telemetry(&self, corridor_id: &str) -> Option<TelemetryData> {
        self.telemetry.get(corridor_id).cloned()
    }
}

impl fmt::Display for HiveInnerLedger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HiveInnerLedger: RoH={}, Adjustments={}", self.roh.0, self.adjustments.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roh_safety() {
        let safe_roh = RoH::new(0.25);
        assert!(safe_roh.is_safe());
        let unsafe_roh = RoH::new(0.4);
        assert!(!unsafe_roh.is_safe());
    }

    #[test]
    fn test_apply_adjustment_success() {
        let mut ledger = HiveInnerLedger::new();
        let adj = Adjustment::IrrigationLevel(50.0);
        let roh = RoH::new(0.2);
        assert!(ledger.apply_adjustment(adj, roh).is_ok());
        assert_eq!(ledger.current_roh(), roh);
    }

    #[test]
    fn test_apply_adjustment_roh_exceeded() {
        let mut ledger = HiveInnerLedger::new();
        let adj = Adjustment::PlantingSchedule("Optimized".to_string());
        let roh = RoH::new(0.4);
        assert_eq!(ledger.apply_adjustment(adj, roh), Err(LedgerError::RoHExceeded));
    }

    #[test]
    fn test_telemetry_operations() {
        let mut ledger = HiveInnerLedger::new();
        let data = TelemetryData {
            brood_viability: 0.9,
            thermoregulation: 1.0,
            forage_stability: 0.85,
        };
        ledger.add_telemetry("corridor_1".to_string(), data.clone());
        assert_eq!(ledger.query_telemetry("corridor_1"), Some(data));
        assert_eq!(ledger.query_telemetry("unknown"), None);
    }
}

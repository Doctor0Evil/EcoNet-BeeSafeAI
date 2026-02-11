use crate::sovereigntycore::invariants::{RoHAxis, NeurorightsFloor, ViabilityKernel};
use crate::evolution::EvolutionProposalRecord;
use crate::aln::AlnParticle;
use std::collections::HashMap;

/// NATURE_BOUNDS: The only legal state transitions are those that preserve the viability kernel.
/// Consent is a ledger event. NATURE_BOUNDS is a mathematical invariant. One cannot override the other.
#[derive(Debug, Clone, PartialEq)]
pub struct NatureBounds {
    pub kernel: ViabilityKernel,
    pub roh_max: f64, // Fixed at 0.3 per your invariant
    pub neurorights: HashMap<NeurorightsFloor, bool>, // e.g., "mental_privacy", "cognitive_liberty"
    pub metabolic_microgrid: Vec<MetabolicConstraint>,
}

#[derive(Debug, Clone)]
pub struct MetabolicConstraint {
    pub axis: String, // e.g., "thermal_load", "neural_energy", "EMF_exposure"
    pub max: f64,     // Absolute upper bound in physiological units
    pub current: f64, // Current observed state
}

impl NatureBounds {
    pub fn new(kernel: ViabilityKernel, roh_max: f64) -> Self {
        assert!(roh_max <= 0.3, "RoH must not exceed 0.3 per sovereign invariant");
        Self {
            kernel,
            roh_max,
            neurorights: HashMap::from([
                (NeurorightsFloor::MentalPrivacy, true),
                (NeurorightsFloor::CognitiveLiberty, true),
                (NeurorightsFloor::IdentityIntegrity, true),
                (NeurorightsFloor::NonManipulation, true),
            ]),
            metabolic_microgrid: vec![],
        }
    }

    /// Returns `true` if the proposal is structurally impossible to enact, regardless of consent.
    /// This is the core gate: if `is_viable()` returns false, the proposal is pruned before any ledger.
    pub fn is_viable(&self, proposal: &EvolutionProposalRecord) -> bool {
        // 1. Check RoH: must not increase and must remain ≤ 0.3
        let roh_after = proposal.roh_after;
        let roh_before = proposal.roh_before;
        if roh_after > roh_before || roh_after > self.roh_max {
            return false;
        }

        // 2. Check Neurorights: must not weaken any floor
        for (floor, enabled) in &self.neurorights {
            if *enabled && proposal.neurorights_weakened.contains(floor) {
                return false;
            }
        }

        // 3. Check Viability Kernel: trajectory must remain inside K
        if !self.kernel.contains(&proposal.cybostate_delta) {
            return false;
        }

        // 4. Check Metabolic Microgrid: no axis exceeds absolute capacity
        for constraint in &self.metabolic_microgrid {
            let new_load = constraint.current + proposal.impact_on_metabolic.get(&constraint.axis).unwrap_or(&0.0);
            if new_load > constraint.max {
                return false;
            }
        }

        // 5. Check Rollback Integrity: rollback must be executable and within T
        if proposal.rollback_time_hours > 24.0 || !proposal.rollback_plan.is_valid() {
            return false;
        }

        true
    }

    /// Apply this bound to a proposal. If invalid, return Err with reason.
    /// Used by sovereigntycore before any multisig or ledger write.
    pub fn enforce(&self, proposal: &EvolutionProposalRecord) -> Result<(), NatureBoundViolation> {
        if self.is_viable(proposal) {
            Ok(())
        } else {
            Err(NatureBoundViolation {
                proposal_id: proposal.id.clone(),
                reason: "Proposal violates NATURE_BOUNDS: outside viability kernel, RoH increase, neurorights erosion, or metabolic overload.",
            })
        }
    }
}

#[derive(Debug)]
pub struct NatureBoundViolation {
    pub proposal_id: String,
    pub reason: String,
}

impl std::fmt::Display for NatureBoundViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NatureBoundViolation[{}]: {}", self.proposal_id, self.reason)
    }
}

impl std::error::Error for NatureBoundViolation {}

/// Example: How to encode a PROMISE as a Tree-of-Life branch
/// This is not a proposal, but a *promise specification* that must be checked against NATURE_BOUNDS before being written to .evolve.jsonl
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Promise {
    pub id: String,
    pub purpose_vector: Vec<f64>, // e.g., [capability_gain: 0.8, eco_benefit: 0.3, contribution_index: 0.7]
    pub promise_bounds: PromiseBounds,
    pub created_at: u64,
    pub creator_id: String, // e.g., did:ion:EiD8J2b3K8k9Q8x9L7m2n4p1q5r6s7t8u9v0w1x2y3z4A5B6C7D8E9F0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromiseBounds {
    pub roh_max_increase: f64, // Must be ≤ 0.0 (monotonic)
    pub neurorights_preserved: Vec<NeurorightsFloor>,
    pub rollback_time_hours: f64,
    pub metabolic_budget: HashMap<String, f64>, // max additional load per axis
}

impl Promise {
    pub fn is_legally_enforceable(&self, bounds: &NatureBounds) -> bool {
        if self.promise_bounds.roh_max_increase > 0.0 {
            return false; // RoH must not increase
        }
        for floor in &self.promise_bounds.neurorights_preserved {
            if !bounds.neurorights.get(floor).unwrap_or(&false) {
                return false;
            }
        }
        if self.promise_bounds.rollback_time_hours > 24.0 {
            return false;
        }
        // Check metabolic budget against kernel’s microgrid
        for (axis, load) in &self.promise_bounds.metabolic_budget {
            if let Some(constraint) = bounds.metabolic_microgrid.iter().find(|c| c.axis == *axis) {
                if load > &constraint.max {
                    return false;
                }
            } else {
                return false; // unknown axis
            }
        }
        true
    }
}
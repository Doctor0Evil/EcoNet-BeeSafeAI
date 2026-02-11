use crate::cyberswarm::bees::{BeeSafetyKernel, BeeMutationConsentFrame};
use crate::cyberswarm::marine::{MarineSafetyKernel, MarineConsentFrame};
use crate::nature_bounds::{NatureBounds, NatureBoundViolation};
use crate::evolution::EvolutionProposalRecord;

pub struct SovereigntyCore {
    pub human_kernel: NatureBounds,
    pub bee_kernel: BeeSafetyKernel,
    pub marine_kernel: MarineSafetyKernel,
}

impl SovereigntyCore {
    pub fn new() -> Self {
        Self {
            human_kernel: NatureBounds::new(ViabilityKernel::new(), 0.3),
            bee_kernel: BeeSafetyKernel::new(),
            marine_kernel: MarineSafetyKernel::new(),
        }
    }

    /// The only entry point for any proposal.
    /// If it fails here, it is NEVER written to .evolve.jsonl.
    pub fn evaluate_proposal(&self, proposal: &EvolutionProposalRecord) -> Result<(), NatureBoundViolation> {
        // 1. Human proposal must pass human kernel
        self.human_kernel.enforce(proposal)?;

        // 2. If proposal affects bees or marine, it must pass their stricter kernels
        if proposal.affects_bees {
            if !self.bee_kernel.allow_proposal(proposal) {
                return Err(NatureBoundViolation {
                    proposal_id: proposal.id.clone(),
                    reason: "Proposal violates BeeSafetyKernel: V_bee increased or sovereignty below 0.2".to_string(),
                });
            }
        }

        if proposal.affects_marine {
            if !self.marine_kernel.allow_proposal(proposal) {
                return Err(NatureBoundViolation {
                    proposal_id: proposal.id.clone(),
                    reason: "Proposal violates MarineSafetyKernel: V_marine increased or sovereignty below 0.2".to_string(),
                });
            }
        }

        // 3. All promises must be encoded as .promise.aln and pass their bounds
        if let Some(promise) = &proposal.promise {
            if !promise.is_legally_enforceable(&self.human_kernel) {
                return Err(NatureBoundViolation {
                    proposal_id: proposal.id.clone(),
                    reason: "Promise violates NATURE_BOUNDS: non-monotonic RoH, weakened neurorights, or metabolic overload".to_string(),
                });
            }
        }

        Ok(())
    }
}
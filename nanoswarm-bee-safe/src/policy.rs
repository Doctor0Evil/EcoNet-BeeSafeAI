use serde::{Serialize, Deserialize};

use crate::pollinator_profile::{PollinatorProfile, ConsentLevel, BioAttachmentMode};
use crate::eco_envelope::EcoEnvelope;

/// Simplified action enumeration for nanoswarm operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    Sense,
    Clean,
    EmitField,
    DepositPayload,
    Hover,
    Transit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub kind: ActionKind,
    /// Local energy demand (D) in normalized units.
    pub energy_demand: f32,
    /// Knowledge factor contribution (K) – how much empirically grounded information this action adds.
    pub knowledge_gain: f32,
    /// Dracula-wave / coercion risk (DW) – normalized behavioral influence risk.
    pub dracula_wave: f32,
}

/// Result of policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyResult {
    Allowed,
    Denied(String),
}

/// Aggregated response metric used for logging and tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetric {
    pub k: f32,
    pub d: f32,
    pub dw: f32,
}

/// Core policy engine wiring K/D/DW + corridor invariants.
pub struct PollinationPolicy {
    /// Hard upper bound on allowed D in pollinator corridors.
    pub max_d_in_bee_zone: f32,
    /// Hard upper bound on DW near pollinators – typically ~0.
    pub max_dw_near_bees: f32,
}

impl PollinationPolicy {
    pub fn new() -> Self {
        Self {
            max_d_in_bee_zone: 0.1,
            max_dw_near_bees: 0.0,
        }
    }

    /// Compute response metric for logging / analysis.
    pub fn evaluate_response_metric(&self, action: &Action) -> ResponseMetric {
        ResponseMetric {
            k: action.knowledge_gain,
            d: action.energy_demand,
            dw: action.dracula_wave,
        }
    }

    /// Guardrail that enforces K–D–DW tradeoffs in favor of bees and host ecosystem.
    pub fn enforce_kddw_guardrails(
        &self,
        rm: &ResponseMetric,
        env: &EcoEnvelope,
        has_pollinator_profile: bool,
    ) -> PolicyResult {
        if has_pollinator_profile && env.is_pollinator_zone {
            if rm.d > self.max_d_in_bee_zone {
                return PolicyResult::Denied("Energy demand D exceeds bee-zone ceiling".into());
            }
            if rm.dw > self.max_dw_near_bees {
                return PolicyResult::Denied("DW (behavioral coercion) non-zero near pollinators".into());
            }
        }
        PolicyResult::Allowed
    }

    /// Main policy evaluation entry point.
    ///
    /// This should be called *before* any nanoswarm node executes the action.
    pub fn check_policy(
        &self,
        action: &Action,
        target_pollinator: Option<&PollinatorProfile>,
        env: &EcoEnvelope,
    ) -> PolicyResult {
        // 1. If target is a protected pollinator, enforce hard interaction bans.
        if let Some(profile) = target_pollinator {
            if !profile.assert_invariants() {
                return PolicyResult::Denied("Pollinator profile invariants violated".into());
            }

            if profile.is_protected
                && matches!(profile.consent_state, ConsentLevel::PollinatorProtected)
            {
                // Forbidden action kinds near bees.
                match action.kind {
                    ActionKind::Clean
                    | ActionKind::EmitField
                    | ActionKind::DepositPayload
                    | ActionKind::Hover => {
                        return PolicyResult::Denied(
                            "Forbidden interaction with PollinatorProtected object".into(),
                        )
                    }
                    ActionKind::Transit | ActionKind::Sense => { /* still allowed, but see K/D/DW */ }
                }

                // Bio-attachment rules.
                if matches!(profile.bio_affinity_mode, BioAttachmentMode::NoBindPollinator) {
                    if matches!(action.kind, ActionKind::Clean | ActionKind::DepositPayload) {
                        return PolicyResult::Denied(
                            "NoBindPollinator forbids any binding / deposition near bees".into(),
                        );
                    }
                }

                // Energetic + emission envelope.
                if action.energy_demand > profile.energy_budget {
                    return PolicyResult::Denied(
                        "Action energy demand exceeds pollinator energy budget".into(),
                    );
                }
            }
        }

        // 2. K/D/DW guardrails, including cybernetic infrastructure emissions.
        let rm = self.evaluate_response_metric(action);
        let kddw = self.enforce_kddw_guardrails(&rm, env, target_pollinator.is_some());
        if let PolicyResult::Denied(_) = kddw {
            return kddw;
        }

        // 3. EcoEnvelope impact ceiling – treat D as local HostBudget proxy here.
        if env.veto_on_impact(rm.d) {
            return PolicyResult::Denied("HostBudget / eco-impact ceiling exceeded".into());
        }

        PolicyResult::Allowed
    }
}

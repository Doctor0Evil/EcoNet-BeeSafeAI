use crate::xrlabgrid::nanopoly::pollinator_swarm::PollinatorAwareNanoswarm;
use crate::xrlabgrid::nanopoly::pollinator_risk::{BeeRiskScore, BeeRiskTier};
use crate::xrlabgrid::nanopoly::pollinator_controller::{SwarmControlDecision};
use crate::store::metrics::ResponseMetric;

/// Result of attempting to certify a swarm configuration for operation.
#[derive(Clone, Debug)]
pub struct EcoCertification {
    pub swarm_id: String,
    pub certified: bool,
    pub bee_risk: BeeRiskScore,
    pub control_decision: SwarmControlDecision,
    pub certification_k: f32,
    pub certification_d: f32,
    pub certification_dw: f32,
}

/// Governance rule: only Tier2/3 may receive certification, and Tier3 earns rewards.
pub fn certify_pollinator_swarm(
    swarm: &PollinatorAwareNanoswarm,
    eco_impact_score: f32,
    control_decision: SwarmControlDecision,
) -> EcoCertification {
    let metric = swarm.compute_response_metric();
    let lifeforce_index = swarm.lifeforce_index(eco_impact_score);
    let bee_risk = BeeRiskScore::from_metric(metric.clone(), lifeforce_index);

    let certified = match bee_risk.tier {
        BeeRiskTier::Tier0_Shutdown | BeeRiskTier::Tier1_MonitorOnly => false,
        BeeRiskTier::Tier2_LowImpact | BeeRiskTier::Tier3_FullService => true,
    } && !control_decision.must_shutdown;

    EcoCertification {
        swarm_id: swarm.id.clone(),
        certified,
        bee_risk,
        control_decision,
        certification_k: metric.knowledge_factor_k,
        certification_d: metric.demand_d,
        certification_dw: metric.draculawave_dw,
    }
}

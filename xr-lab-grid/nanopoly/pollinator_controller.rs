use crate::xrlabgrid::nanopoly::pollinator_swarm::PollinatorAwareNanoswarm;
use crate::xrlabgrid::nanopoly::pollinator_risk::{BeeRiskScore, BeeRiskTier};

#[derive(Clone, Debug)]
pub struct SwarmControlLimits {
    pub max_members_tier1: usize,
    pub max_members_tier2: usize,
    pub max_members_tier3: usize,
    pub max_flight_speed_mps: f32,
    pub max_light_lux: f32,
}

#[derive(Clone, Debug)]
pub struct SwarmControlDecision {
    pub desired_tier: BeeRiskTier,
    pub allowed_members: usize,
    pub max_speed_mps: f32,
    pub max_light_lux: f32,
    pub must_shutdown: bool,
}

pub fn decide_swarm_controls(
    swarm: &PollinatorAwareNanoswarm,
    eco_impact_score: f32,
    limits: &SwarmControlLimits,
) -> (BeeRiskScore, SwarmControlDecision) {
    let metric = swarm.compute_response_metric();
    let lifeforce_index = swarm.lifeforce_index(eco_impact_score);
    let risk = BeeRiskScore::from_metric(metric, lifeforce_index);

    let total_members = swarm.members.len();
    let (allowed_members, max_speed, max_lux, must_shutdown) = match risk.tier {
        BeeRiskTier::Tier0_Shutdown => (0, 0.0, 0.0, true),
        BeeRiskTier::Tier1_MonitorOnly => (
            total_members.min(limits.max_members_tier1),
            0.0,
            limits.max_light_lux * 0.1,
            false,
        ),
        BeeRiskTier::Tier2_LowImpact => (
            total_members.min(limits.max_members_tier2),
            limits.max_flight_speed_mps * 0.5,
            limits.max_light_lux * 0.5,
            false,
        ),
        BeeRiskTier::Tier3_FullService => (
            total_members.min(limits.max_members_tier3),
            limits.max_flight_speed_mps,
            limits.max_light_lux,
            false,
        ),
    };

    let decision = SwarmControlDecision {
        desired_tier: risk.tier.clone(),
        allowed_members,
        max_speed_mps: max_speed,
        max_light_lux: max_lux,
        must_shutdown,
    };

    (risk, decision)
}

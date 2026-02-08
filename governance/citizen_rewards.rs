use crate::governance::ecocert::EcoCertification;

#[derive(Clone, Debug)]
pub struct CitizenRewardEvent {
    pub swarm_id: String,
    pub tokens_delta: i64,     // positive = reward, negative = penalty
    pub reason: String,
}

pub fn compute_citizen_reward(cert: &EcoCertification) -> CitizenRewardEvent {
    if !cert.certified {
        return CitizenRewardEvent {
            swarm_id: cert.swarm_id.clone(),
            tokens_delta: -10,
            reason: "Operation not certified for bee safety.".to_string(),
        };
    }

    let tier = &cert.bee_risk.tier;
    let lf = cert.bee_risk.lifeforce_index;

    // Reward more for high LifeforceIndex and low D/DW.
    let base = if matches!(tier, crate::xrlabgrid::nanopoly::pollinator_risk::BeeRiskTier::Tier3_FullService) {
        50
    } else {
        20
    };

    let lf_bonus = (lf * 50.0) as i64;
    let d_penalty = (cert.certification_d * 30.0) as i64;
    let dw_penalty = (cert.certification_dw * 30.0) as i64;

    let tokens = base + lf_bonus - d_penalty - dw_penalty;

    CitizenRewardEvent {
        swarm_id: cert.swarm_id.clone(),
        tokens_delta: tokens.max(0),
        reason: "Bee-safe pollination service with LifeforceIndex-weighted reward.".to_string(),
    }
}

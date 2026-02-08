use crate::store::metrics::ResponseMetric;
use crate::xrlabgrid::nanopoly::nanoswarm_bee_policy::Nanoswarm;

#[derive(Clone, Debug)]
pub struct BeeLifeEnvelope {
    pub min_lifeforce_bee: f32,
    pub max_allowed_risk: f32,
    pub max_exclusion_ratio: f32,
}

impl BeeLifeEnvelope {
    pub fn enforce(&self, swarm: &Nanoswarm) -> ResponseMetric {
        let metric = swarm.check_bee_safe_policy();

        let ok_lifeforce = metric.knowledge_factor_k >= self.min_lifeforce_bee;
        let ok_d = metric.demand_d <= self.max_allowed_risk;
        let ok_dw = metric.draculawave_dw <= self.max_exclusion_ratio;

        let allowed = ok_lifeforce && ok_d && ok_dw;

        let notes = if allowed {
            "BeeLifeEnvelope: swarm configuration allowed."
        } else {
            "BeeLifeEnvelope: swarm configuration rejected to protect honeybee ecosystem."
        };

        ResponseMetric::new(
            if allowed { metric.knowledge_factor_k } else { metric.knowledge_factor_k * 0.8 },
            metric.demand_d,
            metric.draculawave_dw,
            notes,
        )
    }
}

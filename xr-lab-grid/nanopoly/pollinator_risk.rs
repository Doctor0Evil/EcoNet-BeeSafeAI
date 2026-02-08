use crate::store::metrics::ResponseMetric;

#[derive(Clone, Debug)]
pub enum BeeRiskTier {
    Tier0_Shutdown,     // No operation allowed
    Tier1_MonitorOnly,  // Sensing only, no actuation
    Tier2_LowImpact,    // Limited actuation, reduced density
    Tier3_FullService,  // Full pollination service
}

#[derive(Clone, Debug)]
pub struct BeeRiskScore {
    pub metric: ResponseMetric,
    pub lifeforce_index: f32,
    pub tier: BeeRiskTier,
}

impl BeeRiskScore {
    pub fn from_metric(metric: ResponseMetric, lifeforce_index: f32) -> Self {
        // Example thresholds; tune from field data.
        let tier = if lifeforce_index < 0.3 || metric.draculawave_dw > 0.6 || metric.demand_d > 0.8
        {
            BeeRiskTier::Tier0_Shutdown
        } else if lifeforce_index < 0.5 || metric.draculawave_dw > 0.5 || metric.demand_d > 0.6 {
            BeeRiskTier::Tier1_MonitorOnly
        } else if lifeforce_index < 0.7 || metric.draculawave_dw > 0.4 || metric.demand_d > 0.5 {
            BeeRiskTier::Tier2_LowImpact
        } else {
            BeeRiskTier::Tier3_FullService
        };

        BeeRiskScore {
            metric,
            lifeforce_index,
            tier,
        }
    }
}

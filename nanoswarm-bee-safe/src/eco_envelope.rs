use serde::{Serialize, Deserialize};

/// Environment-level context around a nanoswarm node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoEnvelope {
    /// True if the current region intersects any pollinator corridor / BeeZone.
    pub is_pollinator_zone: bool,

    /// Normalized flowering density (0 = no blossoms, 1 = peak bloom).
    pub flowering_intensity: f32,

    /// Normalized bee activity index (0 = none, 1 = peak foraging).
    pub bee_activity_index: f32,

    /// Maximum allowed eco-impact metric (e.g., HostBudget or Vbee contribution) in this zone.
    pub eco_impact_ceiling: f32,
}

impl EcoEnvelope {
    /// Tier-3 eco-health mode: very low activity, coarse sampling, no aggressive actions.
    pub fn requires_tier3(&self) -> bool {
        self.is_pollinator_zone && self.bee_activity_index >= 0.6
    }

    /// Hard veto when projected eco-impact exceeds the ceiling.
    pub fn veto_on_impact(&self, projected_impact: f32) -> bool {
        projected_impact > self.eco_impact_ceiling
    }
}

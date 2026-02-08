use crate::xrlabgrid::nanopoly::nanopolygon::Nanopolygon;
use crate::xrlabgrid::nanopoly::pollinator::{
    EcoEnvelope, PollinationPolicy, BeeSafetyStatus,
};
use crate::store::metrics::ResponseMetric;

#[derive(Clone, Debug)]
pub struct PollinatorAwareMember {
    pub polygon: Nanopolygon,
    pub basal_power_uW: f64,
    pub acoustic_db_at_10cm: f32,
    pub optical_lux: f32,
    pub local_temp_rise_c: f32,
}

#[derive(Clone, Debug)]
pub struct PollinatorAwareNanoswarm {
    pub id: String,
    pub members: Vec<PollinatorAwareMember>,
    pub eco_envelope: EcoEnvelope,
    pub policy: PollinationPolicy,
}

impl PollinatorAwareNanoswarm {
    pub fn total_energy_uW(&self) -> f64 {
        self.members.iter().map(|m| m.basal_power_uW).sum()
    }

    /// Compute unified K/D/DW metrics for the swarm in the pollination context.
    pub fn compute_response_metric(&self) -> ResponseMetric {
        let total_energy = self.total_energy_uW();
        // Normalize D to 0–1 using a corridor defined by policy.max_host_energy_d.
        let d_norm = (total_energy / 1_000_000.0).min(1.0) as f32; // 1 W == D=1 cap here
        // K high when inside envelope and instrumentation is good – placeholder.
        let k = 0.9_f32;
        // DW as function of acoustic + optical stress vs bee limits – placeholder.
        let mean_db = self
            .members
            .iter()
            .map(|m| m.acoustic_db_at_10cm as f64)
            .sum::<f64>()
            / self.members.len().max(1) as f64;

        let mean_lux = self
            .members
            .iter()
            .map(|m| m.optical_lux as f64)
            .sum::<f64>()
            / self.members.len().max(1) as f64;

        let db_ratio = (mean_db as f32 / self.eco_envelope.pollinator_profile.max_sound_db_at_10cm)
            .min(2.0);
        let lux_ratio = (mean_lux as f32 / self.eco_envelope.pollinator_profile.max_illum_lux)
            .min(2.0);

        let dw = ((db_ratio + lux_ratio) / 4.0).clamp(0.0, 1.0);
        ResponseMetric::new(k, d_norm, dw, "Pollinator-aware swarm aggregate K/D/DW.")
    }

    /// LifeforceIndex for the bee–biosphere envelope, derived from D, DW and eco-load.
    pub fn lifeforce_index(&self, eco_impact_score: f32) -> f32 {
        // Simple mapping: high eco_impact_score reduces LifeforceIndex.
        // In your existing framework LifeforceIndex = weighted bundle of TD, MB, eco, D, DW.
        let metric = self.compute_response_metric();
        let d_penalty = metric.demand_d;
        let dw_penalty = metric.draculawave_dw;
        let eco_penalty = 1.0 - eco_impact_score.clamp(0.0, 1.0);

        (1.0 - 0.4 * d_penalty - 0.3 * dw_penalty - 0.3 * eco_penalty)
            .clamp(0.0, 1.0)
    }

    pub fn evaluate_bee_safety(&self, eco_impact_score: f32) -> BeeSafetyStatus {
        let metric = self.compute_response_metric();
        let lf = self.lifeforce_index(eco_impact_score);

        let mean_db = self
            .members
            .iter()
            .map(|m| m.acoustic_db_at_10cm as f64)
            .sum::<f64>()
            / self.members.len().max(1) as f64;

        let mean_lux = self
            .members
            .iter()
            .map(|m| m.optical_lux as f64)
            .sum::<f64>()
            / self.members.len().max(1) as f64;

        let mean_temp_rise = self
            .members
            .iter()
            .map(|m| m.local_temp_rise_c as f64)
            .sum::<f64>()
            / self.members.len().max(1) as f64;

        let density_safe = true; // plug in your spatial density estimator

        let acoustic_safe = mean_db as f32
            <= self.eco_envelope.pollinator_profile.max_sound_db_at_10cm
                - self.policy.max_acoustic_db_margin;
        let optical_safe = mean_lux as f32
            <= self.eco_envelope.pollinator_profile.max_illum_lux
                - self.policy.max_optical_lux_margin;
        let thermal_safe = mean_temp_rise as f32 <= self.policy.max_temp_rise_c;

        let within_envelope = true; // check member positions vs EcoEnvelope polygons

        let bee_safety_ok = acoustic_safe
            && optical_safe
            && thermal_safe
            && density_safe
            && within_envelope
            && lf >= self.policy.min_lifeforce_index
            && metric.demand_d <= self.policy.max_host_energy_d
            && metric.draculawave_dw <= self.policy.max_coercion_dw
            && metric.knowledge_factor_k >= self.policy.min_knowledge_k;

        BeeSafetyStatus {
            current_metric: metric,
            lifeforce_index: lf,
            within_envelope,
            acoustic_safe,
            optical_safe,
            thermal_safe,
            density_safe,
            bee_safety_ok,
        }
    }
}

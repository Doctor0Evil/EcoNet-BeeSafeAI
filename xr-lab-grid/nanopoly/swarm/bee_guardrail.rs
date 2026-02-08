use crate::store::metrics::ResponseMetric;

/// Normalized K/D/DW plus key state for a nanoswarm segment.
#[derive(Clone, Debug)]
pub struct SwarmState {
    pub knowledge_factor: f32,  // K in [0.0, 1.0]
    pub demand: f32,            // D in [0.0, 1.0]
    pub dracula_wave: f32,      // DW in [0.0, 1.0]
    pub soc_fraction: f32,      // 0.0–1.0 battery SoC
    pub uv_irradiance_w_m2: f32,
    pub imu_drift_deg_per_hr: f32,
    pub polygon_shrink_factor: f32, // applied shrink vs. calm baseline
}

/// Bee-safe thresholds calibrated from 2023–2026 field data.
#[derive(Clone, Debug)]
pub struct BeeThresholds {
    pub min_k_tier1: f32,
    pub max_d_tier1: f32,
    pub max_dw_tier1: f32,
    pub min_soc_tier1: f32,
    pub max_imu_drift_deg_per_hr: f32,
    pub max_uv_w_m2: f32,

    pub min_k_tier2: f32,
    pub max_d_tier2: f32,
    pub max_dw_tier2: f32,
    pub min_soc_tier2: f32,

    /// Maximum allowed shrink without forced abort (e.g. 0.8 = 20% shrink).
    pub min_allowed_shrink_factor: f32,
}

impl Default for BeeThresholds {
    fn default() -> Self {
        Self {
            // Tier 1 == calm, high-fidelity envelope.
            min_k_tier1: 0.90,
            max_d_tier1: 0.50,
            max_dw_tier1: 0.20,
            min_soc_tier1: 0.50,
            max_imu_drift_deg_per_hr: 0.50,  // near lower bound of field drift
            max_uv_w_m2: 800.0,              // just under peak orchard UV band

            // Tier 2 == degraded but acceptable.
            min_k_tier2: 0.75,
            max_d_tier2: 0.70,
            max_dw_tier2: 0.40,
            min_soc_tier2: 0.30,

            // If thermal/SoC forces shrink beyond ~20%, we abort.
            min_allowed_shrink_factor: 0.80,
        }
    }
}

/// Operational tier after evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeeTier {
    Tier1HighFidelity,
    Tier2Degraded,
    Tier3Failsafe,
}

/// Result of guardrail evaluation.
#[derive(Clone, Debug)]
pub struct GuardrailDecision {
    pub tier: BeeTier,
    pub metric: ResponseMetric,
    pub must_abort: bool,
    pub reason: String,
}

/// Core guardrail function: pure, side-effect-free.
pub fn evaluate_swarm_state(state: &SwarmState, th: &BeeThresholds) -> GuardrailDecision {
    // Panic-on-violation: hard fails if shrink exceeds empirical bounds.
    if state.polygon_shrink_factor < th.min_allowed_shrink_factor {
        // DW spikes under extreme shrink by design.
        let metric = ResponseMetric::new(
            0.3,
            0.9,
            0.9,
            "Polygon shrink beyond bee-safe envelope; immediate abort.",
        );
        return GuardrailDecision {
            tier: BeeTier::Tier3Failsafe,
            metric,
            must_abort: true,
            reason: "excessive_shrink".to_string(),
        };
    }

    // Tier selection.
    let (tier, k, d, dw, reason) = if state.knowledge_factor >= th.min_k_tier1
        && state.demand <= th.max_d_tier1
        && state.dracula_wave <= th.max_dw_tier1
        && state.soc_fraction >= th.min_soc_tier1
        && state.imu_drift_deg_per_hr <= th.max_imu_drift_deg_per_hr
        && state.uv_irradiance_w_m2 <= th.max_uv_w_m2
    {
        (
            BeeTier::Tier1HighFidelity,
            state.knowledge_factor,
            state.demand,
            state.dracula_wave,
            "tier1_bee_calm",
        )
    } else if state.knowledge_factor >= th.min_k_tier2
        && state.demand <= th.max_d_tier2
        && state.dracula_wave <= th.max_dw_tier2
        && state.soc_fraction >= th.min_soc_tier2
    {
        (
            BeeTier::Tier2Degraded,
            state.knowledge_factor,
            state.demand,
            state.dracula_wave,
            "tier2_degraded_conditions",
        )
    } else {
        // Tier 3: anything worse automatically collapses to failsafe,
        // and we push DW upward to reflect coercion risk.
        let dw_escalated = (state.dracula_wave + 0.2).clamp(0.0, 1.0);
        (
            BeeTier::Tier3Failsafe,
            state.knowledge_factor.min(0.7),
            state.demand.max(0.7),
            dw_escalated,
            "tier3_failsafe",
        )
    };

    let metric = ResponseMetric::new(k, d, dw, reason);
    let must_abort = matches!(tier, BeeTier::Tier3Failsafe)
        && (state.soc_fraction <= 0.15
            || state.imu_drift_deg_per_hr > 2.0  // near worst-case field drift
            || state.dracula_wave >= 0.8);

    GuardrailDecision {
        tier,
        metric,
        must_abort,
        reason: reason.to_string(),
    }
}

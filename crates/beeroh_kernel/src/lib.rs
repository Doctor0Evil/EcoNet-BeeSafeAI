#![no_std]

use core::cmp::Ordering;

/// Normalized risk coordinates for one hive window.
/// All components must be in [0, 1], where 0 = no added load, 1 = corridor edge.
#[derive(Copy, Clone, Debug)]
pub struct BeeRiskCoords {
    pub r_thermal: f64,   // brood/shell/WBGT thermal load
    pub r_parasite: f64,  // Varroa / pathogen load
    pub r_nutrition: f64, // forage / weight / nutrition risk
    pub r_disturb: f64,   // acoustic / handling / opening disturbance
    pub r_sigma: f64,     // epistemic uncertainty across modalities
}

/// Weights and thresholds for the BeeRoH Lyapunov-like residual.
#[derive(Copy, Clone, Debug)]
pub struct BeeRiskWeights {
    pub w_thermal: f64,
    pub w_parasite: f64,
    pub w_nutrition: f64,
    pub w_disturb: f64,
    pub w_sigma: f64,
    /// Safe residual ceiling: V_bee <= v_safe ⇒ BeeNeuralSafe may be true.
    pub v_safe: f64,
    /// Critical residual ceiling: V_bee <= v_crit ⇒ below irreversible harm band.
    pub v_crit: f64,
    /// Hard coordinate ceiling: max r_x <= r_hard is required for BeeNeuralSafe.
    pub r_hard: f64,
}

/// Per-window summary of risk and BeeNeuralSafe status.
#[derive(Copy, Clone, Debug)]
pub struct BeeRiskSummary {
    pub v_bee: f64,
    pub max_r: f64,
    pub bee_neural_safe: bool,
}

/// Clamp helper.
fn clamp01(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Compute BeeRoH Lyapunov-like residual and BeeNeuralSafe flag.
///
/// V_bee = Σ w_i * r_i^2, i ∈ {thermal, parasite, nutrition, disturb, sigma}.
/// BeeNeuralSafe = (V_bee <= v_safe) && (max r_i <= r_hard).
pub fn compute_bee_roh(coords: &BeeRiskCoords, w: &BeeRiskWeights) -> BeeRiskSummary {
    let rt = clamp01(coords.r_thermal);
    let rp = clamp01(coords.r_parasite);
    let rn = clamp01(coords.r_nutrition);
    let rd = clamp01(coords.r_disturb);
    let rs = clamp01(coords.r_sigma);

    let v = w.w_thermal * rt * rt
        + w.w_parasite * rp * rp
        + w.w_nutrition * rn * rn
        + w.w_disturb * rd * rd
        + w.w_sigma * rs * rs;

    let mut max_r = rt;
    for r in [rp, rn, rd, rs].iter().copied() {
        if r > max_r {
            max_r = r;
        }
    }

    let bee_neural_safe = (v <= w.v_safe) && (max_r <= w.r_hard);

    BeeRiskSummary {
        v_bee: v,
        max_r,
        bee_neural_safe,
    }
}

/// Check Lyapunov-style non-increase: V_{t+1} <= V_t outside the safe interior.
///
/// Returns true if the pair (prev, curr) respects the invariant.
pub fn lyapunov_non_increase(prev: &BeeRiskSummary, curr: &BeeRiskSummary, w: &BeeRiskWeights) -> bool {
    // Only enforce monotone non-increase once we are outside a stricter inner kernel.
    // Inner kernel here is taken as V_bee <= 0.5 * v_safe.
    let inner_kernel = 0.5 * w.v_safe;

    match prev.v_bee.partial_cmp(&inner_kernel) {
        Some(Ordering::Greater) => {
            // Outside inner kernel: require V_{t+1} <= V_t.
            match curr.v_bee.partial_cmp(&prev.v_bee) {
                Some(Ordering::Less) | Some(Ordering::Equal) => true,
                _ => false,
            }
        }
        _ => true, // inside kernel: no monotonicity requirement
    }
}

/// Hard gate for any hive-adjacent actuation, logging, or reward.
///
/// Returns true if and only if BeeNeuralSafe is true *and* Lyapunov non-increase holds.
pub fn permit_actions(
    prev: &BeeRiskSummary,
    curr: &BeeRiskSummary,
    w: &BeeRiskWeights,
) -> bool {
    curr.bee_neural_safe && lyapunov_non_increase(prev, curr, w)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_weights() -> BeeRiskWeights {
        BeeRiskWeights {
            w_thermal: 0.35,
            w_parasite: 0.30,
            w_nutrition: 0.15,
            w_disturb: 0.10,
            w_sigma: 0.10,
            v_safe: 0.10,  // research safe ceiling
            v_crit: 0.30,  // critical band (for diagnostics)
            r_hard: 0.80,  // no coordinate may exceed 0.8 for BeeNeuralSafe
        }
    }

    #[test]
    fn safe_low_risk_is_bee_neural_safe() {
        let w = default_weights();
        let coords = BeeRiskCoords {
            r_thermal: 0.2,
            r_parasite: 0.2,
            r_nutrition: 0.1,
            r_disturb: 0.1,
            r_sigma: 0.1,
        };
        let summary = compute_bee_roh(&coords, &w);
        assert!(summary.bee_neural_safe);
        assert!(summary.v_bee <= w.v_safe);
        assert!(summary.max_r <= w.r_hard);
    }

    #[test]
    fn hard_coordinate_violation_fails_bee_neural_safe() {
        let w = default_weights();
        let coords = BeeRiskCoords {
            r_thermal: 0.9, // above r_hard
            r_parasite: 0.1,
            r_nutrition: 0.1,
            r_disturb: 0.1,
            r_sigma: 0.1,
        };
        let summary = compute_bee_roh(&coords, &w);
        assert!(!summary.bee_neural_safe);
    }

    #[test]
    fn lyapunov_non_increase_enforced_outside_inner_kernel() {
        let w = default_weights();
        let prev = BeeRiskSummary {
            v_bee: 0.12,
            max_r: 0.4,
            bee_neural_safe: false,
        };
        let curr_ok = BeeRiskSummary {
            v_bee: 0.11,
            max_r: 0.35,
            bee_neural_safe: true,
        };
        let curr_bad = BeeRiskSummary {
            v_bee: 0.13,
            max_r: 0.35,
            bee_neural_safe: true,
        };
        assert!(lyapunov_non_increase(&prev, &curr_ok, &w));
        assert!(!lyapunov_non_increase(&prev, &curr_bad, &w));
    }

    #[test]
    fn permit_actions_only_when_safe_and_non_increasing() {
        let w = default_weights();
        let prev = BeeRiskSummary {
            v_bee: 0.12,
            max_r: 0.4,
            bee_neural_safe: false,
        };
        let curr_safe = BeeRiskSummary {
            v_bee: 0.11,
            max_r: 0.3,
            bee_neural_safe: true,
        };
        let curr_unsafe = BeeRiskSummary {
            v_bee: 0.11,
            max_r: 0.9,
            bee_neural_safe: false,
        };
        assert!(permit_actions(&prev, &curr_safe, &w));
        assert!(!permit_actions(&prev, &curr_unsafe, &w));
    }
}

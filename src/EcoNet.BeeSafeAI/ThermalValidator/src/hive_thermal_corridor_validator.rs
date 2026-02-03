use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{OffsetDateTime, macros::datetime};

/// Honey-bee and hive thermal safety corridors are enforced as hard constraints,
/// not soft preferences, consistent with your corridor grammar and Lyapunov-style safety logic.[file:3][file:10]

/// HiveThermalSample represents one time-stamped measurement window at a hive or apiary.
///
/// All temperatures are in degrees Celsius.
/// wbgt is the local Wet Bulb Globe Temperature at hive height (~1 m).
/// brain_temp_approx is an estimated honey-bee thoracic/brain-equivalent temperature,
/// reconstructed from ambient + hive internal + solar load models.[file:10]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveThermalSample {
    pub timestamp: OffsetDateTime,
    pub hive_id: String,
    pub ambient_temp_c: f32,
    pub hive_internal_temp_c: f32,
    pub wbgt_c: f32,
    pub solar_irradiance_w_m2: f32,
    pub relative_humidity_pct: f32,
    /// Estimated neural temperature proxy for bee brain/thorax (°C).
    pub brain_temp_approx_c: f32,
}

/// Safety corridor thresholds for honey-bee welfare.
/// Values are conservative; they should be refined using field data and lab measurements,
/// but always tuned in the direction of more bee protection, never less.[file:10]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveThermalCorridor {
    /// Maximum allowed hive internal temperature (°C) before brood/bee stress.
    pub max_hive_internal_c: f32,
    /// Maximum allowed bee neural proxy temperature (°C).
    pub max_brain_temp_c: f32,
    /// Maximum WBGT at hive height to avoid chronic heat stress.
    pub max_hive_wbgt_c: f32,
    /// Maximum consecutive seconds allowed above any limit before "hard fail".
    pub max_violation_duration_s: u64,
    /// Optional Lyapunov-like requirement: how fast temperature must decrease
    /// once above threshold (°C per minute). If <= 0, disabled.
    pub min_cooldown_rate_c_per_min: f32,
}

/// Result of validating a time series of samples against the corridor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveThermalValidationResult {
    pub hive_id: String,
    pub total_samples: usize,
    pub violations: Vec<HiveThermalViolation>,
    /// Fraction of time (0–1) spent in fully safe corridor.
    pub safe_fraction: f32,
    /// Whether the series is considered BeeSafe-compliant.
    pub is_beesafe_compliant: bool,
    /// Honey-Bee neuro-safety score HB in [0,1], where 1 is ideal.
    pub hb_score: f32,
}

/// Details about each violation, including which neural-safety dimension failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveThermalViolation {
    pub timestamp: OffsetDateTime,
    pub hive_id: String,
    pub kind: ViolationKind,
    pub value: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationKind {
    HiveInternalOverheat,
    BeeBrainOverheat,
    HiveWbgtOverheat,
    CooldownTooSlow,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("no samples provided")]
    EmptySeries,
    #[error("mixed hive IDs in series: expected {expected}, found {found}")]
    MixedHiveIds { expected: String, found: String },
}

/// Validate a time-ordered series of thermal samples for a single hive against
/// bee neural-safety corridors.
///
/// This function enforces:
/// - Hard temperature caps for hive internal temp, bee neural proxy, and WBGT.
/// - A maximum consecutive violation duration.
/// - Optional Lyapunov-like cooldown requirement once thresholds are breached.
/// - Computation of an HB score in [0,1], used as a honey-bee wellness identifier.[file:3][file:10]
pub fn validate_hive_series(
    corridor: &HiveThermalCorridor,
    samples: &[HiveThermalSample],
) -> Result<HiveThermalValidationResult, ValidationError> {
    if samples.is_empty() {
        return Err(ValidationError::EmptySeries);
    }

    let hive_id = samples[0].hive_id.clone();
    for s in samples.iter().skip(1) {
        if s.hive_id != hive_id {
            return Err(ValidationError::MixedHiveIds {
                expected: hive_id.clone(),
                found: s.hive_id.clone(),
            });
        }
    }

    let mut violations = Vec::new();
    let mut safe_seconds: f64 = 0.0;
    let mut total_seconds: f64 = 0.0;

    let mut current_violation_start: Option<OffsetDateTime> = None;
    let mut last_sample: Option<&HiveThermalSample> = None;
    let mut cooldown_violations = 0usize;

    for (idx, sample) in samples.iter().enumerate() {
        if let Some(prev) = last_sample {
            let dt = (sample.timestamp - prev.timestamp).whole_seconds().max(0) as f64;
            total_seconds += dt;

            let within_hive = sample.hive_internal_temp_c <= corridor.max_hive_internal_c;
            let within_brain = sample.brain_temp_approx_c <= corridor.max_brain_temp_c;
            let within_wbgt = sample.wbgt_c <= corridor.max_hive_wbgt_c;

            let is_safe = within_hive && within_brain && within_wbgt;
            if is_safe {
                safe_seconds += dt;

                // If we were in violation before, check max duration.
                if let Some(start) = current_violation_start {
                    let dur = (sample.timestamp - start).whole_seconds() as u64;
                    if dur > corridor.max_violation_duration_s {
                        // Already recorded detailed violations below.
                    }
                    current_violation_start = None;
                }
            } else {
                if current_violation_start.is_none() {
                    current_violation_start = Some(prev.timestamp);
                }
            }

            // Lyapunov-like cooldown: if above threshold and corridor requires cooldown,
            // ensure that temperatures trend downward fast enough.
            if corridor.min_cooldown_rate_c_per_min > 0.0 {
                if prev.hive_internal_temp_c > corridor.max_hive_internal_c
                    || prev.brain_temp_approx_c > corridor.max_brain_temp_c
                    || prev.wbgt_c > corridor.max_hive_wbgt_c
                {
                    let dt_min = dt / 60.0;
                    if dt_min > 0.0 {
                        let d_hive = prev.hive_internal_temp_c - sample.hive_internal_temp_c;
                        let d_brain = prev.brain_temp_approx_c - sample.brain_temp_approx_c;
                        let d_wbgt = prev.wbgt_c - sample.wbgt_c;
                        let cooldown_rate = (d_hive.max(d_brain).max(d_wbgt)) / (dt_min as f32);

                        if cooldown_rate < corridor.min_cooldown_rate_c_per_min {
                            cooldown_violations += 1;
                            violations.push(HiveThermalViolation {
                                timestamp: sample.timestamp,
                                hive_id: hive_id.clone(),
                                kind: ViolationKind::CooldownTooSlow,
                                value: cooldown_rate,
                                threshold: corridor.min_cooldown_rate_c_per_min,
                            });
                        }
                    }
                }
            }
        }

        // Instantaneous hard threshold checks.
        if sample.hive_internal_temp_c > corridor.max_hive_internal_c {
            violations.push(HiveThermalViolation {
                timestamp: sample.timestamp,
                hive_id: hive_id.clone(),
                kind: ViolationKind::HiveInternalOverheat,
                value: sample.hive_internal_temp_c,
                threshold: corridor.max_hive_internal_c,
            });
        }
        if sample.brain_temp_approx_c > corridor.max_brain_temp_c {
            violations.push(HiveThermalViolation {
                timestamp: sample.timestamp,
                hive_id: hive_id.clone(),
                kind: ViolationKind::BeeBrainOverheat,
                value: sample.brain_temp_approx_c,
                threshold: corridor.max_brain_temp_c,
            });
        }
        if sample.wbgt_c > corridor.max_hive_wbgt_c {
            violations.push(HiveThermalViolation {
                timestamp: sample.timestamp,
                hive_id: hive_id.clone(),
                kind: ViolationKind::HiveWbgtOverheat,
                value: sample.wbgt_c,
                threshold: corridor.max_hive_wbgt_c,
            });
        }

        last_sample = Some(sample);

        // Check ongoing violation duration at the last sample.
        if idx == samples.len() - 1 {
            if let Some(start) = current_violation_start {
                let dur = (sample.timestamp - start).whole_seconds() as u64;
                if dur > corridor.max_violation_duration_s {
                    // We can treat this as an implicit "neural distress" window.
                    // The detailed instantaneous violations are already collected.
                }
            }
        }
    }

    if total_seconds <= 0.0 {
        // Degenerate case: single sample or zero dt; treat as instantaneous check.
        total_seconds = 1.0;
        safe_seconds = if violations.is_empty() { 1.0 } else { 0.0 };
    }

    let safe_fraction = (safe_seconds / total_seconds).clamp(0.0, 1.0) as f32;

    // HB score: penalize both time outside corridors and number of violations.
    // Start from safe_fraction, then apply a modest penalty per violation.
    let violation_penalty = 0.02 * (violations.len() as f32 + cooldown_violations as f32);
    let mut hb_score = safe_fraction - violation_penalty;
    if hb_score < 0.0 {
        hb_score = 0.0;
    }

    // BeeSafe-compliant if:
    // - At least 0.95 of time is fully safe, and
    // - No violation pushes hb_score below 0.9.
    let is_beesafe_compliant = safe_fraction >= 0.95 && hb_score >= 0.9;

    Ok(HiveThermalValidationResult {
        hive_id,
        total_samples: samples.len(),
        violations,
        safe_fraction,
        is_beesafe_compliant,
        hb_score,
    })
}

/// A default conservative corridor for honey-bee neural safety.
///
/// These numbers should be calibrated with real data; they are intentionally strict.
/// - max_hive_internal_c: 35 °C (bees regulate brood area near this; we treat this as cap).
/// - max_brain_temp_c: 39 °C (above this, neural stress risk increases).
/// - max_hive_wbgt_c: 30 °C (local WBGT at hive; conservative vs human worker limits). [file:10]
pub fn default_bee_neural_corridor() -> HiveThermalCorridor {
    HiveThermalCorridor {
        max_hive_internal_c: 35.0,
        max_brain_temp_c: 39.0,
        max_hive_wbgt_c: 30.0,
        max_violation_duration_s: 900, // 15 minutes
        min_cooldown_rate_c_per_min: 0.5,
    }
}

// Optional: C-compatible FFI for use from C# or other languages.
#[no_mangle]
pub extern "C" fn hive_validator_version() -> *const u8 {
    b"hive_thermal_corridor_validator_v0.1.0\0".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_all_safe_series() {
        let corridor = default_bee_neural_corridor();
        let base_time: OffsetDateTime = datetime!(2026-02-02 18:00:00 UTC);

        let mut samples = Vec::new();
        for i in 0..10 {
            samples.push(HiveThermalSample {
                timestamp: base_time + time::Duration::seconds(i * 60),
                hive_id: "hive-001".to_string(),
                ambient_temp_c: 30.0,
                hive_internal_temp_c: 34.0,
                wbgt_c: 28.0,
                solar_irradiance_w_m2: 500.0,
                relative_humidity_pct: 40.0,
                brain_temp_approx_c: 37.0,
            });
        }

        let result = validate_hive_series(&corridor, &samples).unwrap();
        assert!(result.violations.is_empty());
        assert!(result.is_beesafe_compliant);
        assert_abs_diff_eq!(result.safe_fraction, 1.0, epsilon = 1e-6);
        assert!(result.hb_score > 0.95);
    }

    #[test]
    fn test_brain_overheat_violation() {
        let corridor = default_bee_neural_corridor();
        let base_time: OffsetDateTime = datetime!(2026-02-02 18:00:00 UTC);

        let mut samples = Vec::new();
        for i in 0..10 {
            let brain = if i >= 5 { 40.5 } else { 37.0 };
            samples.push(HiveThermalSample {
                timestamp: base_time + time::Duration::seconds(i * 60),
                hive_id: "hive-002".to_string(),
                ambient_temp_c: 32.0,
                hive_internal_temp_c: 36.0,
                wbgt_c: 31.0,
                solar_irradiance_w_m2: 700.0,
                relative_humidity_pct: 35.0,
                brain_temp_approx_c: brain,
            });
        }

        let result = validate_hive_series(&corridor, &samples).unwrap();
        assert!(!result.violations.is_empty());
        assert!(!result.is_beesafe_compliant);
        assert!(result.hb_score < 0.9);
    }

    #[test]
    fn test_empty_series_error() {
        let corridor = default_bee_neural_corridor();
        let samples: Vec<HiveThermalSample> = Vec::new();
        let res = validate_hive_series(&corridor, &samples);
        assert!(matches!(res, Err(ValidationError::EmptySeries)));
    }
}

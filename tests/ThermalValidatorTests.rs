use hive_thermal_corridor_validator::{
    default_bee_neural_corridor, validate_hive_series, HiveThermalSample,
};
use time::{OffsetDateTime, macros::datetime};

#[test]
fn hb_score_improves_with_better_conditions() {
    let corridor = default_bee_neural_corridor();
    let base_time: OffsetDateTime = datetime!(2026-02-02 18:00:00 UTC);

    // Scenario A: marginally hot hive.
    let mut samples_hot = Vec::new();
    for i in 0..20 {
        samples_hot.push(HiveThermalSample {
            timestamp: base_time + time::Duration::seconds(i * 60),
            hive_id: "hive-A".to_string(),
            ambient_temp_c: 36.0,
            hive_internal_temp_c: 36.5,
            wbgt_c: 31.5,
            solar_irradiance_w_m2: 800.0,
            relative_humidity_pct: 30.0,
            brain_temp_approx_c: 40.5,
        });
    }

    let res_hot = validate_hive_series(&corridor, &samples_hot).unwrap();

    // Scenario B: cooled hive with shading and ventilation.
    let mut samples_cool = Vec::new();
    for i in 0..20 {
        samples_cool.push(HiveThermalSample {
            timestamp: base_time + time::Duration::seconds(i * 60),
            hive_id: "hive-B".to_string(),
            ambient_temp_c: 32.0,
            hive_internal_temp_c: 34.0,
            wbgt_c: 27.5,
            solar_irradiance_w_m2: 400.0,
            relative_humidity_pct: 40.0,
            brain_temp_approx_c: 37.5,
        });
    }

    let res_cool = validate_hive_series(&corridor, &samples_cool).unwrap();

    assert!(res_cool.hb_score > res_hot.hb_score);
    assert!(res_cool.safe_fraction > res_hot.safe_fraction);
}

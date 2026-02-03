// Hex-stamp: 0xa1b2c3d4e5f67890
// Knowledge-Factor: 0.93, Eco-impact: 0.90, Risk-of-harm: 0.13

pub mod bands {
    /// Corridor bands for a single bee-relevant metric (dimensionless risk 0–1).
    #[derive(Clone, Debug)]
    pub struct CorridorBands {
        pub var_id: &'static str,
        pub units: &'static str,      // e.g., "dimensionless", "C", "ug/m3"
        pub safe: f64,                // safe band upper bound (<= gold)
        pub gold: f64,                // preferred band upper bound (<= hard)
        pub hard: f64,                // hard limit (must not be exceeded)
        pub weight: f64,              // contribution to residual V
        pub lyap_channel: u32,        // for diagnostics
        pub mandatory: bool,          // true => no corridor, no build
    }

    impl CorridorBands {
        pub fn new(
            var_id: &'static str,
            units: &'static str,
            safe: f64,
            gold: f64,
            hard: f64,
            weight: f64,
            lyap_channel: u32,
            mandatory: bool,
        ) -> Self {
            Self {
                var_id,
                units,
                safe,
                gold,
                hard,
                weight,
                lyap_channel,
                mandatory,
            }
        }
    }
}

pub mod risk {
    use super::bands::CorridorBands;

    /// Single normalized risk coordinate r_x in [0, 1] with uncertainty.
    #[derive(Clone, Debug)]
    pub struct RiskCoord {
        pub var_id: &'static str,
        pub value: f64,   // normalized risk coordinate r_x
        pub sigma: f64,   // uncertainty
        pub bands: CorridorBands,
    }

    /// Aggregate residual V_t and decision flags for a hive step.
    #[derive(Clone, Debug)]
    pub struct Residual {
        pub vt: f64,
        pub coords: Vec<RiskCoord>,
        pub derate: bool,
        pub stop: bool,
    }

    /// Piecewise-linear normalization into r_x using safegoldhard bands.
    pub fn to_risk(measured: f64, bands: &CorridorBands) -> f64 {
        if measured <= bands.safe {
            0.0
        } else if measured >= bands.hard {
            1.0
        } else {
            // Map [safe, hard] -> [0, 1]
            (measured - bands.safe) / (bands.hard - bands.safe)
        }
    }

    /// Compute V_t = sum_j w_j * r_j.
    pub fn compute_residual(coords: &[RiskCoord]) -> f64 {
        coords
            .iter()
            .map(|c| c.bands.weight * c.value)
            .sum()
    }
}

pub mod hive {

    use super::bands::CorridorBands;
    use super::risk::{compute_residual, to_risk, Residual, RiskCoord};

    /// Bee-centered envelope: no human fields; only hive and landscape metrics.
    #[derive(Clone, Debug)]
    pub struct HiveEnvelope {
        pub hive_id: String,
        pub region: String,
        // Core hive metrics (raw physical values).
        pub brood_temp_c: f64,
        pub hive_temp_c: f64,
        pub hive_humidity_pct: f64,
        pub nectar_kg: f64,
        pub pollen_kg: f64,
        pub forager_load_pct: f64,       // fraction of foragers vs. sustainable load
        pub toxin_index_air: f64,        // e.g., normalized pesticide index
        pub toxin_index_wax: f64,
        pub forage_radius_km: f64,
        pub eco_band: EcoBand,
    }

    #[derive(Clone, Debug, Copy, PartialEq, Eq)]
    pub enum EcoBand {
        Safe,
        Warning,
        Critical,
    }

    /// Environmental, landscape-level adjustment; never direct bee actuation.
    #[derive(Clone, Debug)]
    pub struct HiveSystemAdjustment {
        pub hive_id: String,
        pub delta_wildflower_area_m2: f64,
        pub delta_pesticide_use_pct: f64,
        pub delta_irrigation_m3_per_day: f64,
        pub delta_light_pollution_lm: f64,
        pub delta_foraging_corridor_km: f64,
        pub rationale: &'static str,
    }

    /// Corridors required for bee safety (temperature, toxins, forage, etc.).
    #[derive(Clone, Debug)]
    pub struct HiveCorridors {
        pub temp_bands: CorridorBands,
        pub brood_temp_bands: CorridorBands,
        pub humidity_bands: CorridorBands,
        pub toxin_air_bands: CorridorBands,
        pub toxin_wax_bands: CorridorBands,
        pub forage_radius_bands: CorridorBands,
        pub forager_load_bands: CorridorBands,
    }

    /// Policy thresholds summarized as KER for the hive corridor state.
    #[derive(Clone, Debug)]
    pub struct HiveKER {
        pub knowledge_factor: f64,   // 0–1 coverage of critical bee variables
        pub eco_impact: f64,         // 0–1 eco benefit kernel
        pub risk_of_harm: f64,       // 0–1 residual corridor penetration
    }

    /// No-corridor, no-build invariant: all mandatory corridors must be present
    /// and well-formed before any hive can be admitted to the governed stack.
    pub fn corridor_present(c: &HiveCorridors) -> bool {
        let bands = [
            &c.temp_bands,
            &c.brood_temp_bands,
            &c.humidity_bands,
            &c.toxin_air_bands,
            &c.toxin_wax_bands,
            &c.forage_radius_bands,
            &c.forager_load_bands,
        ];

        bands.iter().all(|b| {
            (!b.mandatory) || (b.hard > 0.0 && b.gold <= b.hard && b.safe <= b.gold)
        })
    }

    /// Compute hive residual and band (Safe / Warning / Critical).
    pub fn evaluate_hive(env: &HiveEnvelope, corridors: &HiveCorridors) -> Residual {
        let coords = vec![
            RiskCoord {
                var_id: corridors.temp_bands.var_id,
                sigma: 0.05,
                value: to_risk(env.hive_temp_c, &corridors.temp_bands),
                bands: corridors.temp_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.brood_temp_bands.var_id,
                sigma: 0.05,
                value: to_risk(env.brood_temp_c, &corridors.brood_temp_bands),
                bands: corridors.brood_temp_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.humidity_bands.var_id,
                sigma: 0.05,
                value: to_risk(env.hive_humidity_pct, &corridors.humidity_bands),
                bands: corridors.humidity_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.toxin_air_bands.var_id,
                sigma: 0.10,
                value: to_risk(env.toxin_index_air, &corridors.toxin_air_bands),
                bands: corridors.toxin_air_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.toxin_wax_bands.var_id,
                sigma: 0.10,
                value: to_risk(env.toxin_index_wax, &corridors.toxin_wax_bands),
                bands: corridors.toxin_wax_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.forage_radius_bands.var_id,
                sigma: 0.05,
                value: to_risk(env.forage_radius_km, &corridors.forage_radius_bands),
                bands: corridors.forage_radius_bands.clone(),
            },
            RiskCoord {
                var_id: corridors.forager_load_bands.var_id,
                sigma: 0.05,
                value: to_risk(env.forager_load_pct, &corridors.forager_load_bands),
                bands: corridors.forager_load_bands.clone(),
            },
        ];

        let vt = compute_residual(&coords);

        let mut derate = false;
        let mut stop = false;

        for c in &coords {
            if c.value >= 1.0 {
                // Hard violation: hive in critical corridor → stop.
                stop = true;
            } else if c.value > c.bands.gold {
                // Between gold and hard: derate.
                derate = true;
            }
        }

        Residual { vt, coords, derate, stop }
    }

    /// Runtime invariant: no adjustment may increase bee risk or violate hard limits.
    /// This is the "safestep" analogue for hives.
    pub fn safe_step(prev: &Residual, next: &Residual) -> Residual {
        let mut decision = next.clone();

        // Lyapunov monotonicity outside the safe interior.
        if next.vt > prev.vt && prev.coords.iter().any(|c| c.value > 0.0) {
            decision.derate = true;
            decision.stop = true;
        }

        // Any hard-limit violation in next state forces stop.
        for c in &next.coords {
            if c.value >= 1.0 {
                decision.stop = true;
            }
        }

        decision
    }

    /// Example policy: no action may increase pesticide exposure, raise hive
    /// temperature above safe band, or reduce forage radius below corridor.
    pub fn policy_allows_adjustment(
        envelope_before: &HiveEnvelope,
        envelope_after: &HiveEnvelope,
        corridors: &HiveCorridors,
    ) -> bool {
        // Pesticide / toxin invariants (monotone non-increasing).
        if envelope_after.toxin_index_air > envelope_before.toxin_index_air {
            return false;
        }
        if envelope_after.toxin_index_wax > envelope_before.toxin_index_wax {
            return false;
        }

        // Hive temperature must not move from <= safe band to > safe band.
        let r_before_temp = super::risk::to_risk(envelope_before.hive_temp_c, &corridors.temp_bands);
        let r_after_temp  = super::risk::to_risk(envelope_after.hive_temp_c, &corridors.temp_bands);
        if r_before_temp <= corridors.temp_bands.safe && r_after_temp > corridors.temp_bands.safe {
            return false;
        }

        // Forage radius must not shrink below safe band.
        let r_before_forage = super::risk::to_risk(
            envelope_before.forage_radius_km,
            &corridors.forage_radius_bands,
        );
        let r_after_forage = super::risk::to_risk(
            envelope_after.forage_radius_km,
            &corridors.forage_radius_bands,
        );
        if r_after_forage > r_before_forage {
            return false;
        }

        true
    }
}

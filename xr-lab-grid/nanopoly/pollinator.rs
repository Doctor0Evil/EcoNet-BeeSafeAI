use crate::xrlabgrid::nanopoly::nanopolygon::Nanopolygon;
use crate::store::metrics::ResponseMetric;

/// Species-specific sensory and biomechanical bounds for Apis mellifera–class pollinators.
#[derive(Clone, Debug)]
pub struct PollinatorProfile {
    pub species_id: String,          // e.g. "apis_mellifera"
    pub min_wingbeat_hz: f32,       // typical band, e.g. 180.0
    pub max_wingbeat_hz: f32,       // safe ceiling, e.g. 230.0
    pub max_sound_db_at_10cm: f32,  // safe acoustic ceiling at hive entrance
    pub max_illum_lux: f32,         // safe illumination threshold in foraging corridors
    pub max_flight_temp_c: f32,     // upper safe ambient °C
    pub min_flight_temp_c: f32,     // lower safe ambient °C
    pub max_accel_g: f32,           // max maneuver/propwash acceleration near bees
    pub sensor_noise_pos_cm: f32,   // metrology uncertainty, e.g. ±2.7 cm
}

/// Spatial envelope of bee presence and exclusion zones at nanoswarm planning scale.
#[derive(Clone, Debug)]
pub struct EcoEnvelope {
    pub id: String,
    pub region_label: String,       // "orchard_row_7", "urban_corridor_A"
    pub hive_centers_m: Vec<[f32; 3]>,
    pub foraging_corridor_poly: Vec<[f32; 3]>, // coarse polygonal corridor in meters
    pub exclusion_shell_radius_m: f32,         // buffer around hive/corridor
    pub safe_altitude_min_m: f32,
    pub safe_altitude_max_m: f32,
    pub max_swarm_density_per_m3: f32,        // nanoswarm nodes per cubic meter
    pub pollinator_profile: PollinatorProfile,
}

/// Policy struct that encodes bee-first constraints in machine-checkable form.
#[derive(Clone, Debug)]
pub struct PollinationPolicy {
    pub id: String,
    pub eco_envelope_id: String,
    pub max_host_energy_d: f32,        // max normalized D for bees + biosphere
    pub max_coercion_dw: f32,          // DW ceiling for behavioral leverage
    pub min_knowledge_k: f32,          // minimum K to allow operation
    pub min_lifeforce_index: f32,      // LifeforceIndex threshold for beescape
    pub max_acoustic_db_margin: f32,   // nanoswarm must stay this far below bee limit
    pub max_optical_lux_margin: f32,
    pub max_temp_rise_c: f32,          // allowed local ∆T in bee flight layer
    pub max_swarm_density_margin: f32, // fraction of EcoEnvelope.max_swarm_density_per_m3
    pub allow_night_only_ops: bool,
    pub enforce_citizen_reward: bool,
}

/// Nanoswarm-level bee-safety status snapshot.
#[derive(Clone, Debug)]
pub struct BeeSafetyStatus {
    pub current_metric: ResponseMetric,  // K/D/DW for this swarm mode
    pub lifeforce_index: f32,           // eco LifeforceIndex in 0–1
    pub within_envelope: bool,
    pub acoustic_safe: bool,
    pub optical_safe: bool,
    pub thermal_safe: bool,
    pub density_safe: bool,
    pub bee_safety_ok: bool,
}

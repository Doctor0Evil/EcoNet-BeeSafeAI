use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeeRiskCoords {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub z_m: f32,
    pub sigma_lat_m: f32,
    pub sigma_lon_m: f32,
    pub sigma_z_m: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vbee {
    pub eco_credits: f64,
    pub stake_hash: String,
    pub policy_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SensingMode {
    RemoteOptical,
    AcousticExternal,
    EnvironmentalStation,
    Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeeShard {
    pub shard_id: String,
    pub site_id: String,
    pub timestamp_start: String,
    pub timestamp_end: String,
    pub hb_window_hours: f32,

    pub bee_neural_safe: bool,
    pub bee_hb_score: f64,
    pub risk_coords: BeeRiskCoords,
    pub vbee: Vbee,

    pub thermal_c_mean: f32,
    pub thermal_c_min: f32,
    pub thermal_c_max: f32,

    pub chem_aqi: f32,
    pub chem_pesticide_ng_m3: Option<f32>,

    pub emf_ut_mean: f32,
    pub emf_ut_peak: f32,

    pub noise_dba_mean: f32,
    pub noise_dba_peak: f32,

    pub light_lux_mean: f32,
    pub light_lux_peak: f32,

    pub floral_density_units_m2: f32,
    pub diet_diversity_index: f32,

    pub sensor_uq_score: f32,

    pub in_hive_hardware_present: bool,
    pub bee_tagging_used: bool,
    pub sensing_mode: SensingMode,
}

#[derive(Clone, Debug)]
pub enum Medium {
    Air,
    Water,
    Soil,
}

#[derive(Clone, Debug)]
pub enum BeeLifeStage {
    Egg,
    Larva,
    Pupa,
    AdultWorker,
    AdultDrone,
    AdultQueen,
}

#[derive(Clone, Debug)]
pub struct BeeSensitivityProfile {
    pub stage: BeeLifeStage,
    pub thermal_safe_min_c: f32,   // safe ambient band for this stage
    pub thermal_safe_max_c: f32,
    pub chem_ld50_ng_cm3: f32,     // proxy for key toxin / nanoparticle load
    pub rf_safe_max_mw_cm2: f32,   // safe RF / EM flux for this stage
    pub rad_safe_max_uSv_h: f32,   // radiological background envelope
}

#[derive(Clone, Debug)]
pub struct BeeMBIFields {
    pub mbi_mean: f32,       // 0..1 metabolic-molecular balance (stable brood / foraging)
    pub mbi_amp: f32,        // 0..1 oscillation amplitude over window (stress oscillations)
    pub mbi_slope_max: f32,  // max |d(MBI)/dt| over window, normalized
}

#[derive(Clone, Debug)]
pub struct Lifeforce5DVoxelBee {
    // 5D core
    pub t_start_s: f64,
    pub t_end_s: f64,
    pub x_m: f32,
    pub y_m: f32,
    pub z_m: f32,
    pub medium: Medium,

    // Biophysical fields
    pub temp_c: f32,
    pub chem_conc_ng_cm3: f32,
    pub rf_flux_mw_cm2: f32,
    pub rad_uSv_h: f32,

    // Normalized indices 0..1 (1 = healthiest / safest)
    pub thermal_distance: f32,     // TD_bee
    pub molecular_balance: f32,    // MB_bee (hive-level metabolic / chemical stability)
    pub eco_impact_score: f32,     // eco burden of swarm activity in this voxel (lower is better burden; we invert below)
    pub fatigue_index: f32,        // colony stress / fatigue proxy
    pub risk_score: f32,           // combined acute risk (thermal+chem+rad+RF)
    pub mbi: BeeMBIFields,

    // Composite lifeforce for this voxel with respect to bees
    pub lifeforce_index_bee: f32,  // 0..1

    // Exclusion state
    pub bee_sensitivity: BeeSensitivityProfile,
    pub exclusion_active: bool,
}

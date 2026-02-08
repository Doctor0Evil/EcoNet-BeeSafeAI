#[derive(Clone, Debug)]
pub struct NanowaveMBI {
    pub mbi_mean: f32,       // 0..1 average metabolic/molecular balance over window
    pub mbi_amp: f32,        // 0..1 amplitude of oscillation
    pub mbi_slope_max: f32,  // 0..1 max normalized |dMBI/dt|
}

#[derive(Clone, Debug)]
pub struct RadiologicalProfile {
    pub rad_uSv_h: f32,          // measured background dose rate
    pub rad_binding_capacity: f32, // 0..1 fraction of local binding/immobilization capacity in use
    pub rad_immobilization_rate: f32, // 0..1 normalized rate at which swarm immobilizes radionuclides
}

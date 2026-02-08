use crate::store::metrics::ResponseMetric;

#[derive(Clone, Debug)]
pub struct VanillaNanoswarmSnapshot {
    pub id: String,
    pub total_power_uW: f64,
    pub acoustic_db_at_10cm: f32,
    pub optical_lux: f32,
}

impl VanillaNanoswarmSnapshot {
    pub fn to_response_metric(&self) -> ResponseMetric {
        let d_norm = (self.total_power_uW / 1_000_000.0).min(1.0) as f32;
        let k = 0.7_f32; // lower K: no ecological model
        let dw = 0.5_f32; // higher DW: no bee guardrails
        ResponseMetric::new(k, d_norm, dw, "Vanilla nanoswarm baseline K/D/DW.")
    }
}

#[derive(Clone, Debug)]
pub enum ConventionalPollinationType {
    HabitatRestoration,
    FloralStrips,
    RoboticPollinators,
    DronePollenSpray,
}

#[derive(Clone, Debug)]
pub struct ConventionalPollinationSnapshot {
    pub method: ConventionalPollinationType,
    pub mean_power_w: f32,        // e.g. 4.2–6.8 W robo-bees or drones
    pub ecological_disturbance: f32, // proxy for eco-load 0–1
    pub behavior_intrusion: f32,     // proxy for DW analog 0–1
}

impl ConventionalPollinationSnapshot {
    pub fn to_response_metric(&self) -> ResponseMetric {
        let d_norm = (self.mean_power_w / 10.0).min(1.0); // 10 W = D=1
        let k = 0.6_f32; // less fine-grained interaction knowledge
        let dw = self.behavior_intrusion.clamp(0.0, 1.0);
        ResponseMetric::new(k, d_norm, dw, "Conventional pollination K/D/DW analog.")
    }
}

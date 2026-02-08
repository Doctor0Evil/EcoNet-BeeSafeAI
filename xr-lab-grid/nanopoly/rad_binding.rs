use crate::xrlabgrid::nanopoly::nanopoly_object::NanopolyObject;

impl NanopolyObject {
    pub fn can_perform_rad_binding(&self) -> bool {
        let r = &self.env.radiology;
        // Only bind if local capacity is not saturated and bee lifeforce is high enough.
        let capacity_ok = r.rad_binding_capacity < 0.9;
        let bee_ok = self.env.lifeforce_bee.lifeforce_index_bee >= 0.85;
        capacity_ok && bee_ok
    }

    pub fn apply_rad_binding_step(&mut self, delta_uSv_h: f32) {
        if !self.can_perform_rad_binding() {
            return;
        }

        // Reduce free dose rate and increase immobilization capacity usage.
        let new_rad = (self.env.radiology.rad_uSv_h - delta_uSv_h).max(0.0);
        self.env.radiology.rad_uSv_h = new_rad;

        let cap_inc = (delta_uSv_h / (delta_uSv_h + 1.0)).min(0.1);
        self.env.radiology.rad_binding_capacity =
            (self.env.radiology.rad_binding_capacity + cap_inc).clamp(0.0, 1.0);
    }
}

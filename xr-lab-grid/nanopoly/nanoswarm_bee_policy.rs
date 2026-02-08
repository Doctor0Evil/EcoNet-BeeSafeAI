use crate::xrlabgrid::nanopoly::nanopoly_object::NanopolyObject;
use crate::store::metrics::ResponseMetric;

#[derive(Clone, Debug)]
pub struct Nanoswarm {
    pub id: String,
    pub members: Vec<NanopolyObject>,
    pub max_energy_d: f32,
    pub max_dw: f32,
}

impl Nanoswarm {
    pub fn check_bee_safe_policy(&self) -> ResponseMetric {
        // Aggregate host-independent environmental risk to bees.
        let mut worst_lifeforce = 1.0_f32;
        let mut worst_risk = 0.0_f32;
        let mut exclusion_voxel_count = 0_u32;

        for m in &self.members {
            let lf = m.env.lifeforce_bee.lifeforce_index_bee;
            let r = m.env.lifeforce_bee.risk_score;
            worst_lifeforce = worst_lifeforce.min(lf);
            worst_risk = worst_risk.max(r);
            if m.env.lifeforce_bee.exclusion_active {
                exclusion_voxel_count += 1;
            }
        }

        let exclusion_ratio = if self.members.is_empty() {
            0.0
        } else {
            exclusion_voxel_count as f32 / self.members.len() as f32
        };

        // Map environmental risk into D (energy/demand burden) and DW (psych/eco leverage).
        let d = (worst_risk + exclusion_ratio).clamp(0.0, 1.0);
        let dw = (worst_risk * 0.7 + exclusion_ratio * 0.3).clamp(0.0, 1.0);

        let k = (0.5 + 0.5 * worst_lifeforce).clamp(0.0, 1.0);

        ResponseMetric::new(
            k,
            d.min(self.max_energy_d),
            dw.min(self.max_dw),
            "Bee-safe nanoswarm policy evaluation.",
        )
    }

    pub fn is_route_allowed(&self, candidate: &NanopolyObject) -> bool {
        // Hard constraint: never enter a voxel marked exclusion_active.
        if candidate.env.lifeforce_bee.exclusion_active {
            return false;
        }
        // Additional constraint: avoid voxels where lifeforce_index_bee < 0.9 in hive core.
        candidate.env.lifeforce_bee.lifeforce_index_bee >= 0.9
    }
}

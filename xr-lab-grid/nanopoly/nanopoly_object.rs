use crate::xrlabgrid::nanopoly::nanopolygon::Nanopolygon;
use crate::store::metrics::ResponseMetric;
use crate::reality_os::lifeforce5d_voxel_bee::{Lifeforce5DVoxelBee, Medium};
use crate::xrlabgrid::nanopoly::nanowave::{NanowaveMBI, RadiologicalProfile};

#[derive(Clone, Debug)]
pub struct EnvContext {
    pub medium: Medium,
    pub lifeforce_bee: Lifeforce5DVoxelBee,
    pub nanowave_mbi: NanowaveMBI,
    pub radiology: RadiologicalProfile,
}

#[derive(Clone, Debug)]
pub struct NanopolyObject {
    pub id: String,
    pub polygon: Nanopolygon,
    pub env: EnvContext,
    pub metric: ResponseMetric, // K, D, DW
}

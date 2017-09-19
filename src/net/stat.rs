
use ecs::{Component, DenseVecStorage};
use net::NetId;
use net::sync::SyncSeq;

/// Network status component
/// Consists of `NetId` of the entity, `SyncSeq` of last update and `NetId` of the owner
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetStat {
    pub(crate) id: NetId,
    pub(crate) owner: NetId,
    pub(crate) sync_seq: SyncSeq,
}

impl Component for NetStat {
    type Storage = DenseVecStorage<Self>;
}

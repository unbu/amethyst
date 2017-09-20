use ecs::{Component, DenseVecStorage};
use net::{Error, ErrorKind, NetId};
use net::sync::SyncSeq;

/// Network status component
/// Consists of `NetId` of the entity, `SyncSeq` of last update and `NetId` of the owner
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetStat {
    id: NetId,
    owner: NetId,
    sync_seq: SyncSeq,
}

impl NetStat {
    pub(crate) fn new(id: NetId, owner: NetId) -> Self {
        NetStat {
            id: id,
            owner: owner,
            sync_seq: SyncSeq::new(),
        }
    }

    pub(crate) fn update(&mut self, new: NetStat) -> Result<bool, Error> {
        debug_assert_eq!(self.id, new.id);
        if new.owner == self.owner {
            Ok(self.sync_seq.update(new.sync_seq))
        } else {
            Err(ErrorKind::SyncWrongOwner(new.owner, new.id, self.owner).into())
        }
    }

    pub(crate) fn id(&self) -> NetId {
        self.id
    }
}

impl Component for NetStat {
    type Storage = DenseVecStorage<Self>;
}

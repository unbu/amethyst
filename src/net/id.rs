
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NetId(pub(crate) u64);

impl From<NetId> for u64 {
    fn from(id: NetId) -> u64 {
        id.0
    }
}

pub trait NetTypeId {
    const ID: NetId;
}

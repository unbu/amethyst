

/// Defines sequence index of update
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncSeq(pub(crate) u64);

impl SyncSeq {
    pub(crate) fn new() -> Self {
        SyncSeq(0)
    }

    /// Update sequence index
    /// Returns `true` if index is updated
    /// `false` if upcoming update has lesser or equal index
    pub(crate) fn update(&mut self, rhs: SyncSeq) -> bool {
        if self.0 >= rhs.0 {
            false
        } else {
            self.0 = rhs.0;
            true
        }
    }
}

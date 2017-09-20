
use serde::ser::{Serialize, Serializer};

use ecs::{World};

use net::{Error, ErrorKind, NetId, NetStat};
use net::sync::SyncSeq;

impl Serialize for WorldSerializer<'a, R, C>(&'a World) {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.
    }
}


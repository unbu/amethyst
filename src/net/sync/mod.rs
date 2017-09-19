
use serde::{Deserialize, Serialize};
use ecs::{Component, DenseVecStorage};

use net::NetId;

mod de;
mod seq;


pub use self::seq::SyncSeq;
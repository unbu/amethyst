//! Networking library for `Amethyst`

#![allow(missing_docs)]

mod error;
mod id;
mod stat;

pub mod sync;

pub use self::error::{Error, ErrorKind};
pub use self::id::NetId;
pub use self::stat::NetStat;

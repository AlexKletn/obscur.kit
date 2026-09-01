#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod generation;
mod hashing;
mod id;
mod key;

pub use generation::{Generation, VersionStamp};
pub use hashing::{FastBuildHasher, FastHasher, FastMap, FastSet};
pub use id::{
    ComponentId, CorrelationId, FrameId, NamespaceId, NodeId, OperationId, ResourceId, RuntimeId,
    SignalId, WindowId,
};
pub use key::{InternedSymbol, StableKey};

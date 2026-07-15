//! Typed, generational identifiers shared across every Obscur Kit subsystem.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod generation;
mod id;
mod key;

pub use generation::{Generation, VersionStamp};
pub use id::{
    ComponentId, CorrelationId, NamespaceId, NodeId, OperationId, ResourceId, RuntimeId, SignalId,
    WindowId,
};
pub use key::{InternedSymbol, StableKey};

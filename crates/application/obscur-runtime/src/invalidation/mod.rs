#![doc = include_str!("mod.doc/_module.md")]
mod invalidation;
pub use invalidation::Invalidation;
mod invalidation_kind;
pub use invalidation_kind::InvalidationKind;
mod invalidation_reason;
pub use invalidation_reason::InvalidationReason;
mod invalidation_router;
pub use invalidation_router::InvalidationRouter;
mod restyle_scope;
pub use restyle_scope::RestyleScope;

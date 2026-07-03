//! Retained tree and reconciliation — the single owner of UI identity (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod node;

pub use node::RetainedNode;

pub mod layer;
pub mod portal;
pub mod reconciliation;
pub mod transaction;
pub mod tree;

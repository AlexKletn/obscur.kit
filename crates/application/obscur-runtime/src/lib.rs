//! Embeddable `UiRuntime`: frame pipeline, scheduler and UI transactions (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod runtime;

pub use runtime::{FrameBudget, KeyDispatch, PointerDispatch, SemanticTreeBuilder, UiRuntime};

pub mod frame;
pub mod invalidation;
pub mod lifecycle;
pub mod orchestration;
pub mod quality;
pub mod recovery;
pub mod scheduler;
pub mod transaction;

//! Measure/arrange layout contracts and the layout graph (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod model;

pub use model::LayoutBox;

pub mod algorithm;
pub mod context;
pub mod invalidation;
pub mod observer;
pub mod pass;
pub mod port;
pub mod virtualization;

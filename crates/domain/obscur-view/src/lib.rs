//! Component contracts and the desired declarative view tree (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod component;

pub use component::ComponentContract;

pub mod boundary;
pub mod contract;
pub mod lifecycle;
pub mod state;
pub mod view;

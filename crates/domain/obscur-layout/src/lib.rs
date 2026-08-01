#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod model;

pub use model::LayoutBox;

pub mod algorithm;
pub mod context;
pub mod invalidation;
pub mod observer;
pub mod pass;
pub mod port;
pub mod virtualization;

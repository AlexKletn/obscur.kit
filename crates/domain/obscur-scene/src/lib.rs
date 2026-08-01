#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod color;

pub use color::Color;

pub mod clip;
pub mod command;
pub mod layer;
pub mod paint;
pub mod path;
pub mod scene;
pub mod surface;
pub mod text;

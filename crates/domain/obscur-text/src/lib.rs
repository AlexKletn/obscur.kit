#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod editing;
pub mod font;
pub mod ime;
pub mod layout;
pub mod selection;
pub mod service;
pub mod shaping;
pub mod span;

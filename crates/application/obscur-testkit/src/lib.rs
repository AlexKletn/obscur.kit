#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod assertion;
pub mod fixture;
pub mod host;
pub mod mock;
pub mod selector;

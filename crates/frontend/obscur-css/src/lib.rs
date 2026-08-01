#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod cache;
pub mod compiler;
pub mod error;
pub mod parser;
pub mod source;

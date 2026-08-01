#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bridge;
pub mod codegen;
pub mod compile;
pub mod lowering;
pub mod resolution;
pub mod symbol;
pub mod typing;
pub mod validation;

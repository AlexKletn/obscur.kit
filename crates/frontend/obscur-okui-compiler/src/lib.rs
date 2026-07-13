//! .okui compiler: resolution, typing, validation and lowering to UI IR.
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

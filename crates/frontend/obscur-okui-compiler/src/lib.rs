//! .okui compiler: resolution, typing, validation and lowering to UI IR.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bridge;
pub mod codegen;
pub mod compile;
mod compile_diagnostic;
mod compile_result;
mod contract_registry;
pub mod lowering;
pub mod resolution;
pub mod symbol;
pub mod typing;
pub mod validation;

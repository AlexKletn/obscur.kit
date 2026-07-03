//! CSS syntax frontend compiling to obscur-style/obscur-motion structures.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod compiler;
pub mod error;
pub mod parser;
pub mod source;

//! Security-sensitive plugin package reading and validation.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod archive;
pub mod extraction;
pub mod package;
pub mod path;
pub mod validation;

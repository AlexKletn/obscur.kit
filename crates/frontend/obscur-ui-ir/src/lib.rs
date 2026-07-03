//! Versioned, validated UI intermediate representation.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod codec;
pub mod document;
pub mod migration;
pub mod source;
pub mod validation;
pub mod version;

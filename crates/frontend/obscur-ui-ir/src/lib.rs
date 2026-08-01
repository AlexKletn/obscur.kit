#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod codec;
pub mod document;
pub mod migration;
pub mod name_binding;
pub mod source;
pub mod validation;
pub mod version;

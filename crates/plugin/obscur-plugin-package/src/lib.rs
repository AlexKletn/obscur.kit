#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[doc = include_str!("lib.doc/archive.md")]
pub mod archive;
#[doc = include_str!("lib.doc/extraction.md")]
pub mod extraction;
#[doc = include_str!("lib.doc/package.md")]
pub mod package;
#[doc = include_str!("lib.doc/path.md")]
pub mod path;
#[doc = include_str!("lib.doc/validation.md")]
pub mod validation;

#![doc = include_str!("../docs/api/lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod catalog;
/// Locale-aware formatting of values.
pub mod formatting;
pub mod locale;
pub mod message;
pub mod plural;

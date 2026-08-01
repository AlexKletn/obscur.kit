#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod driver;
pub mod phase;
pub mod scheduler;
pub mod timeline;
pub mod track;
pub mod transition;
pub mod value;

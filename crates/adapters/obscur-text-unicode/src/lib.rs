#![doc = include_str!("lib.doc/_module.md")]
#![deny(unsafe_op_in_unsafe_fn)]

mod itemizer;
mod script_mapping;

pub use itemizer::UnicodeTextItemizer;

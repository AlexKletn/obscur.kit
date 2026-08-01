#![doc = include_str!("lib.doc/_module.md")]
#![deny(unsafe_op_in_unsafe_fn)]

mod english_hyphenator;

pub use english_hyphenator::EnglishHyphenator;

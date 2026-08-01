#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod computed;

pub use computed::ComputedStyle;

pub mod cascade;
pub mod invalidation;
pub mod matching;
pub mod property;
pub mod scope;
pub mod selector;

#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod component;

pub mod boundary;
pub mod contract;
pub mod lifecycle;
pub mod state;
pub mod view;

#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod deterministic;
pub mod host;

pub use deterministic::DeterministicClock;
pub use host::HeadlessHost;

pub mod input;
pub mod replay;
pub mod snapshot;

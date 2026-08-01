#![doc = include_str!("mod.doc/_module.md")]
mod clock;
mod deadline;
mod instant;

pub use clock::Clock;
pub use deadline::Deadline;
pub use instant::Instant;

mod duration;

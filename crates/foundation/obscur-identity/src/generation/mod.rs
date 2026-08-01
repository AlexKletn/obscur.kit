#![doc = include_str!("mod.doc/_module.md")]
mod generation;
mod version_stamp;

pub use generation::Generation;
pub use version_stamp::VersionStamp;

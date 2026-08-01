#![doc = include_str!("mod.doc/_module.md")]
mod reader;
pub use reader::read_message;
mod writer;
pub use writer::write_message;

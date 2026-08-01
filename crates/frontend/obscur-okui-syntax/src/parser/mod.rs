#![doc = include_str!("mod.doc/_module.md")]
mod parse_error;
pub use parse_error::{ParseError, ParseErrorKind};
mod parse_event;
pub use parse_event::ParseEvent;
mod parse_result;
pub use parse_result::ParseResult;
mod parser;
pub use parser::Parser;

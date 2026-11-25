//! Parser: an error-tolerant recursive-descent parser that turns a `.okui`
//! [`Token`](crate::lexer::Token) stream into a lossless
//! [`SyntaxTree`](crate::syntax::SyntaxTree) plus recoverable diagnostics.
mod parse_error;
pub use parse_error::ParseError;
mod parse_error_kind;
pub use parse_error_kind::ParseErrorKind;
mod parse_event;
pub use parse_event::ParseEvent;
mod parse_result;
pub use parse_result::ParseResult;
mod parser;
pub use parser::Parser;

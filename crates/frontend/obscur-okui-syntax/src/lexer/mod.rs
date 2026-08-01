#![doc = include_str!("mod.doc/_module.md")]
mod lexer;
pub use lexer::Lexer;
mod token;
pub use token::Token;
mod token_kind;
pub use token_kind::TokenKind;

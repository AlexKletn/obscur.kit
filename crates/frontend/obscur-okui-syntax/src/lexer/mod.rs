//! Lexer: tokenizes `.okui` source into a lossless [`Token`] stream.
mod lexer;
pub use lexer::Lexer;
mod token;
pub use token::Token;
mod token_kind;
pub use token_kind::TokenKind;

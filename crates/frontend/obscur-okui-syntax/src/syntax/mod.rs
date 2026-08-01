#![doc = include_str!("mod.doc/_module.md")]
mod syntax_kind;
pub use syntax_kind::SyntaxKind;
mod syntax_node;
pub use syntax_node::{SyntaxElement, SyntaxNode};
mod syntax_token;
pub use syntax_token::SyntaxToken;
mod syntax_tree;
pub use syntax_tree::SyntaxTree;

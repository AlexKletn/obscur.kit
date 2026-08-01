#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod syntax;

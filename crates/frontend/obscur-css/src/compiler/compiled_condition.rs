use crate::parser::{ParsedContainerQuery, ParsedMediaQueryList};

#[doc = include_str!("../../docs/api/compiler/stylesheet_compiler.doc/CompiledCondition.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledCondition {
    /// An `@media` condition.
    Media(ParsedMediaQueryList),
    /// An `@container` condition.
    Container(ParsedContainerQuery),
}

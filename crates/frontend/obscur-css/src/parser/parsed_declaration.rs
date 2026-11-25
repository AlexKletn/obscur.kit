use crate::source::SourceSpan;

use super::ParsedValue;

/// A single parsed CSS declaration (`color: red !important`).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDeclaration {
    /// The property name (verbatim, so `--custom` keeps its leading dashes).
    pub name: Box<str>,
    /// The parsed component-value list.
    pub value: Vec<ParsedValue>,
    /// Whether the declaration carried a trailing `!important`.
    pub important: bool,
    /// The span covering the whole declaration.
    pub span: SourceSpan,
}

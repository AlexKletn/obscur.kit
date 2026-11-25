use super::ParsedKeyframe;

/// An `@keyframes` rule: a name and its ordered keyframes.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframesRule {
    /// The animation name.
    pub name: Box<str>,
    /// The keyframes, in source order.
    pub keyframes: Vec<ParsedKeyframe>,
}

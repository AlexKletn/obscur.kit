use super::{ParsedContainerRule, ParsedKeyframesRule, ParsedMediaRule, ParsedStyleRule};

/// A single top-level (or nested) CSS rule.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedRule {
    /// A style rule: a selector list and its declaration block.
    Style(ParsedStyleRule),
    /// An `@media` rule with a nested rule list.
    Media(ParsedMediaRule),
    /// An `@container` rule with a nested rule list.
    Container(ParsedContainerRule),
    /// An `@keyframes` rule.
    Keyframes(ParsedKeyframesRule),
}

use super::{ParsedMediaQueryList, ParsedRule};

/// An `@media` rule: a media-query list wrapping a nested rule list.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMediaRule {
    /// The media-query list from the rule's prelude.
    pub query: ParsedMediaQueryList,
    /// The nested rules the media block applies to.
    pub rules: Vec<ParsedRule>,
}

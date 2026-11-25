use super::ParsedMediaQuery;

/// A comma-separated list of media queries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQueryList(pub Vec<ParsedMediaQuery>);

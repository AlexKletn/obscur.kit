use super::{MediaCondition, MediaLogic, MediaQualifier};

/// A single media query: an optional qualifier and media type followed by zero
/// or more feature conditions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedMediaQuery {
    /// The `not`/`only` qualifier, if present.
    pub qualifier: Option<MediaQualifier>,
    /// The media type (`screen`, `all`, `print`, …), if present.
    pub media_type: Option<Box<str>>,
    /// The feature conditions, in source order.
    pub conditions: Vec<MediaCondition>,
    /// The parsed expression over the feature list, if the query contains conditions.
    pub logic: Option<MediaLogic>,
}

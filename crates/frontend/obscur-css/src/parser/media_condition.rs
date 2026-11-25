use super::{MediaComparator, ParsedValue};

/// A single media feature condition (`(min-width: 600px)` or `(width >= 600px)`).
#[derive(Debug, Clone, PartialEq)]
pub struct MediaCondition {
    /// The feature name (for example `min-width`, `prefers-color-scheme`).
    pub feature: Box<str>,
    /// The range comparator, or [`None`] for a colon or boolean feature.
    pub comparator: Option<MediaComparator>,
    /// The feature value, or [`None`] for a boolean feature such as `(hover)`.
    pub value: Option<Vec<ParsedValue>>,
}

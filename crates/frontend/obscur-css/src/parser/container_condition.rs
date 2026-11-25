use super::{MediaComparator, MediaCondition, ParsedValue};

/// A single container feature condition, mirroring the media condition shape.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerCondition {
    /// Whether this feature reads a computed style property instead of a size.
    pub style: bool,
    /// The feature name (for example `min-width`, `inline-size`).
    pub feature: Box<str>,
    /// The range comparator, or [`None`] for a colon or boolean feature.
    pub comparator: Option<MediaComparator>,
    /// The feature value, or [`None`] for a boolean feature.
    pub value: Option<Vec<ParsedValue>>,
}

impl From<MediaCondition> for ContainerCondition {
    fn from(condition: MediaCondition) -> Self {
        Self {
            style: false,
            feature: condition.feature,
            comparator: condition.comparator,
            value: condition.value,
        }
    }
}

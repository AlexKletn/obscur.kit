/// The environment dimension a [`FeatureQuery`](super::FeatureQuery) tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionAxis {
    /// The viewport width (`@media` `width`/`min-width`/`max-width`).
    ViewportWidth,
    /// The viewport height (`@media` `height`/`min-height`/`max-height`).
    ViewportHeight,
    /// The querying container's inline size (`@container` `inline-size`/`width`).
    ContainerWidth,
    /// The querying container's block size (`@container` `block-size`/`height`).
    ContainerHeight,
    /// Logical inline size of the query container.
    ContainerInlineSize,
    /// Logical block size of the query container.
    ContainerBlockSize,
}

impl ConditionAxis {
    #[doc = include_str!("../../docs/api/matching/rule_condition.doc/is_container.md")]
    #[must_use]
    pub const fn is_container(self) -> bool {
        self.container_index().is_some()
    }

    pub(super) const fn container_index(self) -> Option<usize> {
        match self {
            Self::ContainerWidth => Some(0),
            Self::ContainerHeight => Some(1),
            Self::ContainerInlineSize => Some(2),
            Self::ContainerBlockSize => Some(3),
            _ => None,
        }
    }
}

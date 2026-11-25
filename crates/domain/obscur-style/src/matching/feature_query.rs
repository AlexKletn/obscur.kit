use super::{ConditionAxis, ConditionComparator, MediaEnvironment};

#[doc = include_str!("../../docs/api/matching/rule_condition.doc/FeatureQuery.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeatureQuery {
    axis: ConditionAxis,
    comparator: ConditionComparator,
    reference_px: f32,
}

impl FeatureQuery {
    /// Creates a feature query comparing `axis` against `reference_px`.
    #[must_use]
    pub const fn new(
        axis: ConditionAxis,
        comparator: ConditionComparator,
        reference_px: f32,
    ) -> Self {
        Self {
            axis,
            comparator,
            reference_px,
        }
    }

    /// The axis this query tests.
    #[must_use]
    pub const fn axis(&self) -> ConditionAxis {
        self.axis
    }

    /// The comparator this query applies.
    #[must_use]
    pub const fn comparator(&self) -> ConditionComparator {
        self.comparator
    }

    /// The reference length in logical pixels.
    #[must_use]
    pub const fn reference_px(&self) -> f32 {
        self.reference_px
    }

    #[doc = include_str!("../../docs/api/matching/rule_condition.doc/holds.md")]
    #[must_use]
    pub fn holds(&self, env: &MediaEnvironment) -> bool {
        let actual = match self.axis {
            ConditionAxis::ViewportWidth => env.viewport_width(),
            ConditionAxis::ViewportHeight => env.viewport_height(),
            axis => axis
                .container_index()
                .and_then(|index| env.container_size(index)),
        };
        let Some(actual) = actual else {
            return !self.axis.is_container();
        };
        self.comparator.compare(actual, self.reference_px)
    }
}

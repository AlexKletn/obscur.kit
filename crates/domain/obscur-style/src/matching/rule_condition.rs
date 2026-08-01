#![doc = include_str!("rule_condition.doc/_module.md")]

use crate::matching::MediaEnvironment;

#[doc = include_str!("rule_condition.doc/ConditionAxis.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionAxis {
    #[doc = include_str!("rule_condition.doc/ViewportWidth.md")]
    ViewportWidth,
    #[doc = include_str!("rule_condition.doc/ViewportHeight.md")]
    ViewportHeight,
    #[doc = include_str!("rule_condition.doc/ContainerWidth.md")]
    ContainerWidth,
    #[doc = include_str!("rule_condition.doc/ContainerHeight.md")]
    ContainerHeight,
}

impl ConditionAxis {
    #[doc = include_str!("rule_condition.doc/is_container.md")]
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(self, Self::ContainerWidth | Self::ContainerHeight)
    }
}

#[doc = include_str!("rule_condition.doc/ConditionComparator.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionComparator {
    #[doc = include_str!("rule_condition.doc/Lt.md")]
    Lt,
    #[doc = include_str!("rule_condition.doc/Le.md")]
    Le,
    #[doc = include_str!("rule_condition.doc/Eq.md")]
    Eq,
    #[doc = include_str!("rule_condition.doc/Ge.md")]
    Ge,
    #[doc = include_str!("rule_condition.doc/Gt.md")]
    Gt,
}

#[doc = include_str!("rule_condition.doc/FeatureQuery.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeatureQuery {
    axis: ConditionAxis,
    comparator: ConditionComparator,
    reference_px: f32,
}

impl FeatureQuery {
    #[doc = include_str!("rule_condition.doc/new.md")]
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

    #[doc = include_str!("rule_condition.doc/axis.md")]
    #[must_use]
    pub const fn axis(&self) -> ConditionAxis {
        self.axis
    }

    #[doc = include_str!("rule_condition.doc/comparator.md")]
    #[must_use]
    pub const fn comparator(&self) -> ConditionComparator {
        self.comparator
    }

    #[doc = include_str!("rule_condition.doc/reference_px.md")]
    #[must_use]
    pub const fn reference_px(&self) -> f32 {
        self.reference_px
    }

    #[doc = include_str!("rule_condition.doc/holds.md")]
    #[must_use]
    pub fn holds(&self, env: &MediaEnvironment) -> bool {
        let actual = match self.axis {
            ConditionAxis::ViewportWidth => env.viewport_width(),
            ConditionAxis::ViewportHeight => env.viewport_height(),
            ConditionAxis::ContainerWidth => env.container_width(),
            ConditionAxis::ContainerHeight => env.container_height(),
        };
        let Some(actual) = actual else {
            return !self.axis.is_container();
        };
        match self.comparator {
            ConditionComparator::Lt => actual < self.reference_px,
            ConditionComparator::Le => actual <= self.reference_px,
            ConditionComparator::Eq => (actual - self.reference_px).abs() < f32::EPSILON,
            ConditionComparator::Ge => actual >= self.reference_px,
            ConditionComparator::Gt => actual > self.reference_px,
        }
    }
}

#[doc = include_str!("rule_condition.doc/RuleCondition.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum RuleCondition {
    #[doc = include_str!("rule_condition.doc/Feature.md")]
    Feature(FeatureQuery),
    #[doc = include_str!("rule_condition.doc/All.md")]
    All(Vec<RuleCondition>),
    #[doc = include_str!("rule_condition.doc/Any.md")]
    Any(Vec<RuleCondition>),
    #[doc = include_str!("rule_condition.doc/Not.md")]
    Not(Box<RuleCondition>),
    #[doc = include_str!("rule_condition.doc/Always.md")]
    Always,
    #[doc = include_str!("rule_condition.doc/Never.md")]
    Never,
}

impl RuleCondition {
    #[doc = include_str!("rule_condition.doc/holds_2.md")]
    #[must_use]
    pub fn holds(&self, env: &MediaEnvironment) -> bool {
        match self {
            Self::Feature(query) => query.holds(env),
            Self::All(children) => children.iter().all(|child| child.holds(env)),
            Self::Any(children) => children.iter().any(|child| child.holds(env)),
            Self::Not(inner) => !inner.holds(env),
            Self::Always => true,
            Self::Never => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn max_width(px: f32) -> RuleCondition {
        RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ViewportWidth,
            ConditionComparator::Le,
            px,
        ))
    }

    fn min_width(px: f32) -> RuleCondition {
        RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ViewportWidth,
            ConditionComparator::Ge,
            px,
        ))
    }

    #[test]
    fn max_width_gates_on_viewport() {
        let narrow = MediaEnvironment::viewport(1.0, 1000.0);
        let wide = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(max_width(1.0).holds(&narrow));
        assert!(!max_width(1.0).holds(&wide));
    }

    #[test]
    fn unknown_axis_is_non_restrictive() {
        // Unbounded environment: the width axis is unknown, so even a
        // `max-width: 1px` query holds.
        assert!(max_width(1.0).holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn all_requires_every_child() {
        let env = MediaEnvironment::viewport(500.0, 500.0);
        let both = RuleCondition::All(vec![min_width(400.0), max_width(600.0)]);
        assert!(both.holds(&env));
        let broken = RuleCondition::All(vec![min_width(400.0), max_width(499.0)]);
        assert!(!broken.holds(&env));
    }

    #[test]
    fn any_requires_one_child() {
        let env = MediaEnvironment::viewport(500.0, 500.0);
        let either = RuleCondition::Any(vec![max_width(100.0), min_width(400.0)]);
        assert!(either.holds(&env));
        let none = RuleCondition::Any(vec![max_width(100.0), min_width(600.0)]);
        assert!(!none.holds(&env));
    }

    #[test]
    fn not_inverts() {
        let env = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(RuleCondition::Not(Box::new(max_width(1.0))).holds(&env));
    }

    #[test]
    fn always_and_never() {
        let env = MediaEnvironment::viewport(1000.0, 1000.0);
        assert!(RuleCondition::Always.holds(&env));
        assert!(!RuleCondition::Never.holds(&env));
        // Empty All holds; empty Any does not.
        assert!(RuleCondition::All(Vec::new()).holds(&env));
        assert!(!RuleCondition::Any(Vec::new()).holds(&env));
    }

    #[test]
    fn unknown_container_axis_fails_closed() {
        // The regression this guards: with no container size supplied, a
        // `@container (min-width: 300px)` rule used to apply everywhere.
        let container_min = RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ContainerWidth,
            ConditionComparator::Ge,
            300.0,
        ));
        assert!(!container_min.holds(&MediaEnvironment::unbounded()));
        // A known viewport does not make the container axis known.
        assert!(!container_min.holds(&MediaEnvironment::viewport(1000.0, 1000.0)));
        // Supplying the container size is what turns the query back on.
        assert!(
            container_min
                .holds(&MediaEnvironment::viewport(1000.0, 1000.0).with_container(400.0, 100.0))
        );
    }

    #[test]
    fn unknown_container_axis_fails_closed_for_max_queries_too() {
        // A `max-width` query is the dangerous direction: failing OPEN would let
        // it hold, and failing closed must not depend on the comparator.
        let container_max = RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ContainerHeight,
            ConditionComparator::Le,
            300.0,
        ));
        assert!(!container_max.holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn unknown_viewport_axis_stays_non_restrictive() {
        // The asymmetry: viewport axes keep the permissive opt-out.
        assert!(max_width(1.0).holds(&MediaEnvironment::unbounded()));
    }

    #[test]
    fn container_axes_are_independent() {
        let env = MediaEnvironment::viewport(1000.0, 1000.0).with_container(200.0, 150.0);
        let container_min = RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ContainerWidth,
            ConditionComparator::Ge,
            300.0,
        ));
        assert!(!container_min.holds(&env));
        let container_max = RuleCondition::Feature(FeatureQuery::new(
            ConditionAxis::ContainerWidth,
            ConditionComparator::Le,
            300.0,
        ));
        assert!(container_max.holds(&env));
    }
}

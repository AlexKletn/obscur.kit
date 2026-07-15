//! The evaluable `@media`/`@container` condition gating a
//! [`StyleRule`](crate::matching::StyleRule): [`RuleCondition`].
//!
//! This is `obscur-style`'s own, CSS-syntax-free model of a conditional group's
//! condition. The CSS frontend lowers a parsed media/container query into one of
//! these, attaches it to the matchable rule, and the
//! [`StyleMatcher`](crate::matching::StyleMatcher) evaluates it against a
//! [`MediaEnvironment`] at match time — so a conditional rule contributes only
//! when its condition holds.

use crate::matching::MediaEnvironment;

/// The environment dimension a [`FeatureQuery`] tests.
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
}

impl ConditionAxis {
    /// Whether this axis measures a query container rather than the viewport.
    ///
    /// The two families differ in how an *unknown* value is treated — see
    /// [`FeatureQuery::holds`].
    #[must_use]
    pub const fn is_container(self) -> bool {
        matches!(self, Self::ContainerWidth | Self::ContainerHeight)
    }
}

/// The comparison a [`FeatureQuery`] applies between the environment value and
/// its reference length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionComparator {
    /// Strictly less than (`<`).
    Lt,
    /// Less than or equal (`<=`, and the `max-*` prefix forms).
    Le,
    /// Equal (`=`, and a plain `(width: 600px)` colon form).
    Eq,
    /// Greater than or equal (`>=`, and the `min-*` prefix forms).
    Ge,
    /// Strictly greater than (`>`).
    Gt,
}

/// A single feature comparison: `axis comparator reference_px`.
///
/// The reference length is stored resolved to logical pixels. An unknown axis
/// resolves per-family: unknown *viewport* axes are non-restrictive, unknown
/// *container* axes never hold. See [`holds`](Self::holds).
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

    /// Evaluates the query against `env`.
    ///
    /// An unknown axis resolves by family, and the asymmetry is deliberate:
    ///
    /// - An unknown **viewport** axis is non-restrictive (`true`). A live
    ///   runtime always supplies a viewport, so `None` here only ever means a
    ///   caller opted out of gating via [`MediaEnvironment::unbounded`].
    /// - An unknown **container** axis never holds (`false`). `None` means no
    ///   ancestor established a query container, and css-contain-3 §3.1 makes a
    ///   container query against a nonexistent container *false*. Holding here
    ///   would make every `@container` rule apply at every container size —
    ///   which is exactly the bug this asymmetry exists to prevent.
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

/// An evaluable `@media`/`@container` condition tree.
///
/// Built by the CSS frontend from a parsed query and evaluated by the matcher.
/// The frontend maps a *media* feature it cannot represent (a boolean feature, a
/// non-`px` length, an unknown feature name) to [`Always`](Self::Always) so that
/// such rules are never silently dropped; the same unrepresentable feature in a
/// *container* query maps to [`Never`](Self::Never) instead, because no element
/// can currently be a query container (see [`FeatureQuery::holds`]).
#[derive(Debug, Clone, PartialEq)]
pub enum RuleCondition {
    /// A single feature comparison.
    Feature(FeatureQuery),
    /// Conjunction: holds when every child holds (an empty `All` holds).
    All(Vec<RuleCondition>),
    /// Disjunction: holds when any child holds (an empty `Any` never holds).
    Any(Vec<RuleCondition>),
    /// Negation of the inner condition.
    Not(Box<RuleCondition>),
    /// Always holds (an unconstrained `screen` query, or a non-restrictive but
    /// unrepresentable feature).
    Always,
    /// Never holds (for example a `print`-only query in a screen environment).
    Never,
}

impl RuleCondition {
    /// Evaluates the condition against `env`.
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
        assert!(container_min.holds(
            &MediaEnvironment::viewport(1000.0, 1000.0).with_container(400.0, 100.0)
        ));
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

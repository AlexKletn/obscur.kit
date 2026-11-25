#![doc = include_str!("../../docs/api/matching/rule_condition.doc/_module.md")]

use obscur_identity::InternedSymbol;

use super::{
    ConditionAxis, ConditionComparator, FeatureQuery, MediaEnvironment, PointerAccuracy,
    StylePropertyValue, StyleRangeBound,
};
use crate::property::{PropertyId, PropertyValue};

#[doc = include_str!("../../docs/api/matching/rule_condition.doc/RuleCondition.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum RuleCondition {
    /// A style query over a custom property on the selected container.
    ContainerStyle {
        /// The case-sensitive custom property symbol.
        property: InternedSymbol,
        /// The expected value, or no value for a presence query.
        value: Option<PropertyValue>,
    },
    /// A numerical range comparison of a custom property on the selected container.
    ContainerStyleRange {
        /// The case-sensitive custom property symbol.
        property: InternedSymbol,
        /// The operation comparing the property against the reference.
        comparator: ConditionComparator,
        /// The comparison threshold.
        reference: StyleRangeBound,
    },
    /// Two comparisons of a single custom property on the selected container.
    ContainerStyleChain {
        /// The case-sensitive custom property symbol.
        property: InternedSymbol,
        /// The comparison operators and thresholds of both bounds.
        bounds: [(ConditionComparator, StyleRangeBound); 2],
    },
    /// A test of a supported computed CSS longhand on a style container.
    ContainerProperty {
        /// The longhand property to test.
        property: PropertyId,
        /// The expected computed value, or no value to test against the initial.
        value: Option<StylePropertyValue>,
    },
    /// A single feature comparison.
    Feature(FeatureQuery),
    /// Equals the screen's dark color scheme preference.
    PrefersDark(bool),
    /// Matches landscape when the viewport is wider than it is tall. A square viewport is portrait.
    Landscape(bool),
    /// A primary or aggregate pointer accuracy query.
    Pointer {
        /// Include secondary pointers as well as the primary pointer.
        any: bool,
        /// Restrict the query to this accuracy when supplied.
        accuracy: Option<PointerAccuracy>,
        /// Test for presence when true and absence when false.
        present: bool,
    },
    /// A primary or aggregate hover capability query.
    Hover {
        /// Include secondary pointers as well as the primary pointer.
        any: bool,
        /// Match support for hovering when true and lack of support when false.
        enabled: bool,
    },
    /// Compares the viewport hardware pixel ratio to a reference in dppx.
    Resolution(ConditionComparator, f32),
    /// Equals the screen's reduced motion preference.
    PrefersReducedMotion(bool),
    /// Equals the screen's increased contrast preference.
    PrefersIncreasedContrast(bool),
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
    /// An unsupported feature that remains unknown under negation. A known size axis still affects container selection.
    Unknown(Option<ConditionAxis>),
    /// Evaluates a condition using the nearest eligible container ancestor, optionally identified by name.
    Container {
        /// Optional name of the eligible ancestor.
        name: Option<Box<str>>,
        /// Condition evaluated against the selected ancestor.
        condition: Box<RuleCondition>,
    },
}

impl RuleCondition {
    /// Evaluates the condition against `env`.
    #[must_use]
    pub fn holds(&self, env: &MediaEnvironment) -> bool {
        self.evaluate(env) == Some(true)
    }

    fn evaluate(&self, env: &MediaEnvironment) -> Option<bool> {
        match self {
            Self::Feature(query) => Some(query.holds(env)),
            Self::PrefersDark(value) => Some(env.preferences().dark == *value),
            Self::Pointer {
                any,
                accuracy,
                present,
            } => env.input().map(|input| {
                use PointerAccuracy::{Coarse, Fine};
                let matches = match accuracy {
                    Some(accuracy) => input.has_pointer(*accuracy, *any),
                    None => input.has_pointer(Coarse, *any) || input.has_pointer(Fine, *any),
                };
                matches == *present
            }),
            Self::Hover { any, enabled } => {
                env.input().map(|input| input.can_hover(*any) == *enabled)
            }
            Self::Resolution(comparator, reference) => env
                .resolution_dppx()
                .map(|actual| comparator.compare(actual, *reference)),
            Self::Landscape(landscape) => env
                .viewport_width()
                .zip(env.viewport_height())
                .map(|(width, height)| (width > height) == *landscape),
            Self::PrefersReducedMotion(value) => Some(env.preferences().reduced_motion == *value),
            Self::PrefersIncreasedContrast(value) => {
                Some(env.preferences().increased_contrast == *value)
            }
            Self::ContainerStyle { property, value } => {
                Some(env.matches_container_style(*property, value.as_ref()))
            }
            Self::ContainerStyleRange {
                property,
                comparator,
                reference,
            } => {
                let value = env
                    .container_style_range(*property)?
                    .numeric()
                    .and_then(|value| value.resolve(env));
                Some(
                    value
                        .zip(reference.resolve(env))
                        .is_some_and(|(actual, bound)| actual.compare(*comparator, &bound)),
                )
            }
            Self::ContainerStyleChain {
                property,
                bounds: [(left_cmp, left), (right_cmp, right)],
            } => {
                let value = env
                    .container_style_range(*property)?
                    .numeric()
                    .and_then(|value| value.resolve(env));
                let result = value
                    .zip(left.resolve(env))
                    .zip(right.resolve(env))
                    .is_some_and(|((actual, left), right)| {
                        left.same_dimension(&right)
                            && actual.compare(*left_cmp, &left)
                            && actual.compare(*right_cmp, &right)
                    });
                Some(result)
            }
            Self::ContainerProperty { property, value } => {
                env.test_container_property(*property, value.as_ref())
            }
            Self::All(children) => Self::evaluate_group(children, env, false),
            Self::Any(children) => Self::evaluate_group(children, env, true),
            Self::Not(inner) => inner.evaluate(env).map(|result| !result),
            Self::Always => Some(true),
            Self::Never => Some(false),
            Self::Unknown(_) => None,
            Self::Container { name, condition } => {
                let mut required = [false; 4];
                condition.require_container_axes(&mut required);
                let container = env.resolve_container(
                    name.as_deref(),
                    required,
                    condition.uses_container_style(),
                )?;
                condition.evaluate(&container)
            }
        }
    }

    fn evaluate_group(children: &[Self], env: &MediaEnvironment, decisive: bool) -> Option<bool> {
        let mut incomplete = false;
        for child in children {
            match child.evaluate(env) {
                Some(value) if value == decisive => return Some(decisive),
                Some(_) => {}
                None => incomplete = true,
            }
        }
        (!incomplete).then_some(!decisive)
    }

    /// Returns whether this condition needs a container style snapshot.
    #[must_use]
    pub fn uses_container_style(&self) -> bool {
        match self {
            Self::ContainerStyle { .. }
            | Self::ContainerProperty { .. }
            | Self::ContainerStyleRange { .. }
            | Self::ContainerStyleChain { .. } => true,
            Self::All(children) | Self::Any(children) => {
                children.iter().any(Self::uses_container_style)
            }
            Self::Not(inner)
            | Self::Container {
                condition: inner, ..
            } => inner.uses_container_style(),
            _ => false,
        }
    }

    pub(super) fn collect_container_properties(
        &self,
        properties: &mut Vec<PropertyId>,
        ranges: &mut Vec<InternedSymbol>,
        references: &mut Vec<(PropertyId, InternedSymbol)>,
        needs_ch: &mut bool,
    ) {
        match self {
            Self::ContainerProperty { property, value } => {
                if !properties.contains(property) {
                    properties.push(*property);
                }
                if let Some(value) = value {
                    for name in value.variables() {
                        let reference = (*property, name);
                        if !references.contains(&reference) {
                            references.push(reference);
                        }
                    }
                    *needs_ch |= value.terminal_literal().is_some_and(PropertyValue::uses_ch);
                }
            }
            Self::ContainerStyleRange {
                property,
                reference,
                ..
            } => {
                record_range_property(ranges, *property);
                record_range_bound(ranges, reference);
                *needs_ch |= reference.uses_ch();
            }
            Self::ContainerStyleChain { property, bounds } => {
                record_range_property(ranges, *property);
                for (_, bound) in bounds {
                    record_range_bound(ranges, bound);
                    *needs_ch |= bound.uses_ch();
                }
            }
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.collect_container_properties(properties, ranges, references, needs_ch);
                }
            }
            Self::Not(inner)
            | Self::Container {
                condition: inner, ..
            } => {
                inner.collect_container_properties(properties, ranges, references, needs_ch);
            }
            _ => {}
        }
    }

    fn require_container_axes(&self, required: &mut [bool; 4]) {
        match self {
            Self::Feature(query) => {
                if let Some(index) = query.axis().container_index() {
                    required[index] = true;
                }
            }
            Self::Unknown(Some(axis)) => {
                if let Some(index) = axis.container_index() {
                    required[index] = true;
                }
            }
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.require_container_axes(required);
                }
            }
            Self::Not(inner) => inner.require_container_axes(required),
            _ => {}
        }
    }
}

fn record_range_property(ranges: &mut Vec<InternedSymbol>, name: InternedSymbol) {
    if !ranges.contains(&name) {
        ranges.push(name);
    }
}

fn record_range_bound(ranges: &mut Vec<InternedSymbol>, bound: &StyleRangeBound) {
    for name in bound.variables() {
        record_range_property(ranges, name);
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

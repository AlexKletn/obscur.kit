//! A single authored style rule (a selector list plus a declaration block):
//! [`StyleRule`].

use crate::matching::{BlockDeclaration, RuleCondition};
use crate::property::PropertyId;
use crate::selector::SelectorList;

/// One authored style rule: a [`SelectorList`] paired with the block of
/// [`BlockDeclaration`]s it applies.
///
/// This mirrors a CSS qualified rule `selectors { block }`. The rule carries no
/// cascade context of its own; when an element matches the rule's selectors, the
/// [`StyleMatcher`](crate::matching::StyleMatcher) turns each authored
/// declaration in the block into a fully-contextualised
/// [`Declaration`](crate::cascade::Declaration).
///
/// A rule that was authored inside an `@media`/`@container` group additionally
/// carries the group's [`RuleCondition`]; the matcher only lets it contribute
/// when that condition [`holds`](RuleCondition::holds) against the runtime
/// [`MediaEnvironment`](crate::matching::MediaEnvironment). An unconditional
/// rule has no condition and always applies.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    selectors: SelectorList,
    block: Vec<BlockDeclaration>,
    condition: Option<RuleCondition>,
    /// The longhand properties whose authored value could not be parsed and was
    /// therefore dropped from [`block`](Self::block) (§4.13). An element that
    /// matches this rule carries a recoverable, per-node style fault for each —
    /// the value never reached the cascade, so the rest of the block still
    /// applies. Empty for a rule every declaration of which lowered cleanly.
    invalid_values: Vec<PropertyId>,
}

impl StyleRule {
    /// Creates an unconditional rule from its selector list and declaration
    /// block.
    #[must_use]
    pub const fn new(selectors: SelectorList, block: Vec<BlockDeclaration>) -> Self {
        Self {
            selectors,
            block,
            condition: None,
            invalid_values: Vec::new(),
        }
    }

    /// Returns a copy of this rule that records `properties` as the longhands
    /// whose authored value could not be parsed and was dropped (§4.13),
    /// replacing any previously recorded set.
    #[must_use]
    pub fn with_invalid_values(mut self, properties: Vec<PropertyId>) -> Self {
        self.invalid_values = properties;
        self
    }

    /// Returns the longhand properties whose authored value was dropped as
    /// unparseable, in authored order. Empty when every declaration lowered
    /// cleanly.
    #[must_use]
    pub fn invalid_values(&self) -> &[PropertyId] {
        &self.invalid_values
    }

    /// Returns a copy of this rule gated behind `condition`, replacing any
    /// existing condition.
    #[must_use]
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Returns the rule's selector list.
    #[must_use]
    pub const fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    /// Returns the rule's declaration block in authored order.
    #[must_use]
    pub fn block(&self) -> &[BlockDeclaration] {
        &self.block
    }

    /// Returns the `@media`/`@container` condition gating this rule, or [`None`]
    /// for an unconditional rule (which always applies).
    #[must_use]
    pub const fn condition(&self) -> Option<&RuleCondition> {
        self.condition.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{PropertyId, PropertyValue, StyleColor};
    use crate::selector::{ComplexSelector, CompoundSelector, SimpleSelector};

    fn class_selectors(class: &str) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class(class.into())]),
            Vec::new(),
        )])
    }

    #[test]
    fn accessors_expose_parts() {
        let selectors = class_selectors("foo");
        let block = vec![BlockDeclaration::normal(
            PropertyId::Color,
            PropertyValue::Color(StyleColor::BLACK),
        )];
        let rule = StyleRule::new(selectors, block);
        assert_eq!(rule.selectors().selectors().len(), 1);
        assert_eq!(rule.block().len(), 1);
        assert_eq!(rule.block()[0].property(), PropertyId::Color);
        // A rule built with `new` carries no condition.
        assert!(rule.condition().is_none());
    }

    #[test]
    fn with_condition_attaches_and_replaces() {
        use crate::matching::{ConditionAxis, ConditionComparator, FeatureQuery, RuleCondition};
        let rule = StyleRule::new(class_selectors("foo"), Vec::new()).with_condition(
            RuleCondition::Feature(FeatureQuery::new(
                ConditionAxis::ViewportWidth,
                ConditionComparator::Le,
                1.0,
            )),
        );
        assert!(rule.condition().is_some());
        // A second `with_condition` replaces the first.
        let replaced = rule.with_condition(RuleCondition::Never);
        assert_eq!(replaced.condition(), Some(&RuleCondition::Never));
    }
}

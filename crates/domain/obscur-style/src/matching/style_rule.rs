#![doc = include_str!("style_rule.doc/_module.md")]

use crate::matching::{BlockDeclaration, RuleCondition};
use crate::property::PropertyId;
use crate::selector::SelectorList;

#[doc = include_str!("style_rule.doc/StyleRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    selectors: SelectorList,
    block: Vec<BlockDeclaration>,
    condition: Option<RuleCondition>,
    #[doc = include_str!("style_rule.doc/invalid_values.md")]
    invalid_values: Vec<PropertyId>,
}

impl StyleRule {
    #[doc = include_str!("style_rule.doc/new.md")]
    #[must_use]
    pub const fn new(selectors: SelectorList, block: Vec<BlockDeclaration>) -> Self {
        Self {
            selectors,
            block,
            condition: None,
            invalid_values: Vec::new(),
        }
    }

    #[doc = include_str!("style_rule.doc/with_invalid_values.md")]
    #[must_use]
    pub fn with_invalid_values(mut self, properties: Vec<PropertyId>) -> Self {
        self.invalid_values = properties;
        self
    }

    #[doc = include_str!("style_rule.doc/invalid_values_2.md")]
    #[must_use]
    pub fn invalid_values(&self) -> &[PropertyId] {
        &self.invalid_values
    }

    #[doc = include_str!("style_rule.doc/with_condition.md")]
    #[must_use]
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    #[doc = include_str!("style_rule.doc/selectors.md")]
    #[must_use]
    pub const fn selectors(&self) -> &SelectorList {
        &self.selectors
    }

    #[doc = include_str!("style_rule.doc/block.md")]
    #[must_use]
    pub fn block(&self) -> &[BlockDeclaration] {
        &self.block
    }

    #[doc = include_str!("style_rule.doc/condition.md")]
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

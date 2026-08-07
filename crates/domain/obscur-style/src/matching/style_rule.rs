#![doc = include_str!("style_rule.doc/_module.md")]

use crate::matching::{BlockDeclaration, RuleCondition};
use crate::property::{CustomProperty, PropertyId};
use crate::selector::SelectorList;

#[doc = include_str!("style_rule.doc/StyleRule.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    selectors: SelectorList,
    block: Vec<BlockDeclaration>,
    condition: Option<RuleCondition>,
    #[doc = include_str!("style_rule.doc/invalid_values.md")]
    invalid_values: Vec<PropertyId>,
    #[doc = include_str!("style_rule.doc/custom_properties.md")]
    custom_properties: Vec<CustomProperty>,
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
            custom_properties: Vec::new(),
        }
    }

    #[doc = include_str!("style_rule.doc/with_custom_properties.md")]
    #[must_use]
    pub fn with_custom_properties(mut self, properties: Vec<CustomProperty>) -> Self {
        self.custom_properties = properties;
        self
    }

    #[doc = include_str!("style_rule.doc/custom_properties_2.md")]
    #[must_use]
    pub fn custom_properties(&self) -> &[CustomProperty] {
        &self.custom_properties
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
    fn custom_properties_are_carried_beside_the_block_not_inside_it() {
        // They are keyed by name, so the block — keyed by `PropertyId` — has no
        // slot for them, and a rule that dropped them here is exactly how
        // `var()` ends up resolving to nothing.
        use crate::property::CustomProperty;
        use obscur_identity::InternedSymbol;

        let rule = StyleRule::new(class_selectors("foo"), Vec::new()).with_custom_properties(vec![
            CustomProperty::new(
                InternedSymbol::named("--brand"),
                PropertyValue::Color(StyleColor::BLACK),
            ),
        ]);
        assert!(rule.block().is_empty());
        assert_eq!(rule.custom_properties().len(), 1);
        assert_eq!(
            rule.custom_properties()[0].name(),
            InternedSymbol::named("--brand")
        );
        // A rule built with `new` carries none.
        assert!(
            StyleRule::new(class_selectors("foo"), Vec::new())
                .custom_properties()
                .is_empty()
        );
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

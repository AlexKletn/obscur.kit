//! A single authored style rule (a selector list plus a declaration block):
//! [`StyleRule`].

use crate::matching::BlockDeclaration;
use crate::selector::SelectorList;

/// One authored style rule: a [`SelectorList`] paired with the block of
/// [`BlockDeclaration`]s it applies.
///
/// This mirrors a CSS qualified rule `selectors { block }`. The rule carries no
/// cascade context of its own; when an element matches the rule's selectors, the
/// [`StyleMatcher`](crate::matching::StyleMatcher) turns each authored
/// declaration in the block into a fully-contextualised
/// [`Declaration`](crate::cascade::Declaration).
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    selectors: SelectorList,
    block: Vec<BlockDeclaration>,
}

impl StyleRule {
    /// Creates a rule from its selector list and declaration block.
    #[must_use]
    pub const fn new(selectors: SelectorList, block: Vec<BlockDeclaration>) -> Self {
        Self { selectors, block }
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
    }
}

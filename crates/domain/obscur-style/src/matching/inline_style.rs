#![doc = include_str!("inline_style.doc/_module.md")]

use std::sync::Arc;

use crate::matching::BlockDeclaration;
use crate::property::CustomProperty;

#[doc = include_str!("inline_style.doc/InlineStyle.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InlineStyle {
    #[doc = include_str!("inline_style.doc/declarations.md")]
    declarations: Arc<[BlockDeclaration]>,
    #[doc = include_str!("inline_style.doc/custom_properties.md")]
    custom_properties: Arc<[CustomProperty]>,
}

impl InlineStyle {
    #[doc = include_str!("inline_style.doc/new.md")]
    #[must_use]
    pub fn new(declarations: impl Into<Arc<[BlockDeclaration]>>) -> Self {
        Self {
            declarations: declarations.into(),
            custom_properties: Arc::from([]),
        }
    }

    #[doc = include_str!("inline_style.doc/with_custom_properties.md")]
    #[must_use]
    pub fn with_custom_properties(mut self, properties: impl Into<Arc<[CustomProperty]>>) -> Self {
        self.custom_properties = properties.into();
        self
    }

    #[doc = include_str!("inline_style.doc/custom_properties_2.md")]
    #[must_use]
    pub fn custom_properties(&self) -> &[CustomProperty] {
        &self.custom_properties
    }

    #[doc = include_str!("inline_style.doc/empty.md")]
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[doc = include_str!("inline_style.doc/declarations_2.md")]
    #[must_use]
    pub fn declarations(&self) -> &[BlockDeclaration] {
        &self.declarations
    }

    #[doc = include_str!("inline_style.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        // A block that declares only `--brand: #f00` declares SOMETHING; callers
        // skip an empty block, and one that answered `true` here would drop the
        // custom property on the floor.
        self.declarations.is_empty() && self.custom_properties.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::InlineStyle;
    use crate::matching::BlockDeclaration;
    use crate::property::{PropertyId, PropertyValue};

    #[test]
    fn an_empty_block_is_empty() {
        assert!(InlineStyle::empty().is_empty());
        assert!(InlineStyle::empty().declarations().is_empty());
    }

    #[test]
    fn declarations_are_kept_in_the_order_they_were_written() {
        // Source order decides a tie inside one block, so the order the author
        // wrote must survive: `color:red;color:blue` is blue.
        let block = InlineStyle::new(vec![
            BlockDeclaration::normal(PropertyId::Color, PropertyValue::Number(1.0)),
            BlockDeclaration::normal(PropertyId::Color, PropertyValue::Number(2.0)),
        ]);
        assert_eq!(
            block.declarations()[1].value(),
            &PropertyValue::Number(2.0),
            "the later one is still later"
        );
    }

    #[test]
    fn a_block_that_declares_only_a_custom_property_is_not_empty() {
        // The caller skips an empty block, so answering `true` here would be a
        // silent way to lose `style="--brand: red"` entirely.
        use crate::property::CustomProperty;
        use obscur_identity::InternedSymbol;

        let block = InlineStyle::new(Vec::new()).with_custom_properties(vec![CustomProperty::new(
            InternedSymbol::named("--brand"),
            PropertyValue::Number(1.0),
        )]);
        assert!(!block.is_empty());
        assert!(block.declarations().is_empty());
        assert_eq!(block.custom_properties().len(), 1);
    }

    #[test]
    fn cloning_shares_rather_than_duplicates() {
        // A thousand rows of a virtualized list carrying the same block must not
        // be a thousand copies of it.
        let block = InlineStyle::new(vec![BlockDeclaration::normal(
            PropertyId::Color,
            PropertyValue::Number(1.0),
        )]);
        let clone = block.clone();
        assert!(std::ptr::eq(
            std::ptr::from_ref(&block.declarations()[0]),
            std::ptr::from_ref(&clone.declarations()[0])
        ));
    }
}

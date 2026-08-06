#![doc = include_str!("inline_style.doc/_module.md")]

use std::sync::Arc;

use crate::matching::BlockDeclaration;

#[doc = include_str!("inline_style.doc/InlineStyle.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InlineStyle {
    #[doc = include_str!("inline_style.doc/declarations.md")]
    declarations: Arc<[BlockDeclaration]>,
}

impl InlineStyle {
    #[doc = include_str!("inline_style.doc/new.md")]
    #[must_use]
    pub fn new(declarations: impl Into<Arc<[BlockDeclaration]>>) -> Self {
        Self {
            declarations: declarations.into(),
        }
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
        self.declarations.is_empty()
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

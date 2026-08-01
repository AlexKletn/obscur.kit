#![doc = include_str!("style_invalidation_reason.doc/_module.md")]

#[doc = include_str!("style_invalidation_reason.doc/StyleInvalidationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleInvalidationReason {
    #[doc = include_str!("style_invalidation_reason.doc/StylesheetChanged.md")]
    StylesheetChanged,
    #[doc = include_str!("style_invalidation_reason.doc/DeclarationsChanged.md")]
    DeclarationsChanged,
    #[doc = include_str!("style_invalidation_reason.doc/ParentStyleChanged.md")]
    ParentStyleChanged,
    #[doc = include_str!("style_invalidation_reason.doc/CustomPropertyChanged.md")]
    CustomPropertyChanged,
    #[doc = include_str!("style_invalidation_reason.doc/PseudoStateChanged.md")]
    PseudoStateChanged,
    #[doc = include_str!("style_invalidation_reason.doc/InlineStyleChanged.md")]
    InlineStyleChanged,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn is_copy_and_eq() {
        let a = StyleInvalidationReason::ParentStyleChanged;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn is_hashable() {
        let mut set = HashSet::new();
        set.insert(StyleInvalidationReason::StylesheetChanged);
        set.insert(StyleInvalidationReason::StylesheetChanged);
        set.insert(StyleInvalidationReason::PseudoStateChanged);
        assert_eq!(set.len(), 2);
    }
}

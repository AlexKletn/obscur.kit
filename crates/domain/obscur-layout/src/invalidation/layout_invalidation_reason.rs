#![doc = include_str!("layout_invalidation_reason.doc/_module.md")]

#[doc = include_str!("layout_invalidation_reason.doc/LayoutInvalidationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutInvalidationReason {
    #[doc = include_str!("layout_invalidation_reason.doc/StyleChanged.md")]
    StyleChanged,
    #[doc = include_str!("layout_invalidation_reason.doc/TreeChanged.md")]
    TreeChanged,
    #[doc = include_str!("layout_invalidation_reason.doc/ChildSizeChanged.md")]
    ChildSizeChanged,
    #[doc = include_str!("layout_invalidation_reason.doc/ConstraintsChanged.md")]
    ConstraintsChanged,
    #[doc = include_str!("layout_invalidation_reason.doc/ContentChanged.md")]
    ContentChanged,
    #[doc = include_str!("layout_invalidation_reason.doc/Requested.md")]
    Requested,
    #[doc = include_str!("layout_invalidation_reason.doc/EnvironmentChanged.md")]
    EnvironmentChanged,
}

#[cfg(test)]
mod tests {
    use super::LayoutInvalidationReason;

    #[test]
    fn variants_are_distinct() {
        assert_ne!(
            LayoutInvalidationReason::StyleChanged,
            LayoutInvalidationReason::TreeChanged
        );
        assert_ne!(
            LayoutInvalidationReason::ChildSizeChanged,
            LayoutInvalidationReason::ConstraintsChanged
        );
        assert_ne!(
            LayoutInvalidationReason::ContentChanged,
            LayoutInvalidationReason::EnvironmentChanged
        );
    }

    #[test]
    fn is_copy_and_eq() {
        let r = LayoutInvalidationReason::StyleChanged;
        let copied = r;
        assert_eq!(r, copied);
    }

    #[test]
    fn usable_as_hash_map_value() {
        use std::collections::HashMap;
        let mut m: HashMap<u8, LayoutInvalidationReason> = HashMap::new();
        m.insert(1, LayoutInvalidationReason::ConstraintsChanged);
        assert_eq!(
            m.get(&1),
            Some(&LayoutInvalidationReason::ConstraintsChanged)
        );
    }
}

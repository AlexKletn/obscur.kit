//! The typed reason a layout context must be recomputed.

#[doc = include_str!("../../docs/api/invalidation/layout_invalidation_reason.doc/LayoutInvalidationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutInvalidationReason {
    /// A layout-affecting style on the node itself changed (e.g. size, margins,
    /// flex parameters). The node's own layout must be recomputed.
    StyleChanged,
    /// The tree structure under the node changed: children were added or
    /// removed.
    TreeChanged,
    /// A depended-on child's output geometry changed, so this context — whose
    /// measure consumes that child's size — must be recomputed. This is the
    /// carrier of **upward** propagation.
    ChildSizeChanged,
    /// The incoming constraints for this context changed, and its size depends
    /// on them. This is the carrier of **downward** propagation.
    ConstraintsChanged,
    /// The node's intrinsic/text content changed (e.g. a text run relayout),
    /// which can change its measured size without a style or constraint change.
    ContentChanged,
    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation_reason.doc/Requested.md")]
    Requested,
    /// The window environment changed: viewport size, DPI/scale factor, or other
    /// global layout inputs.
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

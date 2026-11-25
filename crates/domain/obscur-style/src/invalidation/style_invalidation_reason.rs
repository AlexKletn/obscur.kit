#![doc = include_str!("../../docs/api/invalidation/style_invalidation_reason.doc/_module.md")]

#[doc = include_str!("../../docs/api/invalidation/style_invalidation_reason.doc/StyleInvalidationReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleInvalidationReason {
    /// A stylesheet was added, removed or mutated, so the node's matched rules
    /// may have changed and it must be re-matched and recomputed.
    StylesheetChanged,
    /// The node's own set of matched declarations changed (for example a rule
    /// that targets it gained or lost a declaration), so it must recompute.
    DeclarationsChanged,
    /// An *inherited* value on the parent's computed style changed, so a value
    /// this node inherited from its parent is now stale (inheritance flows down).
    ParentStyleChanged,
    /// A custom property this node consumed (e.g. via `var()`) changed on an
    /// ancestor, so any value substituted from it must be recomputed.
    CustomPropertyChanged,
    /// The node's active interaction/state pseudo-classes changed (e.g. `:hover`
    /// toggled), so state-dependent selectors must be re-matched.
    PseudoStateChanged,
    /// The node's inline style declarations changed, so it must recompute.
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

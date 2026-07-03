//! Why a node must be restyled: the typed [`StyleInvalidationReason`] (§4.3).
//!
//! Style invalidation in Obscur Kit 2 is dependency-driven with *typed*
//! invalidation reasons, not a scattering of ad-hoc dirty booleans (§4.3). Each
//! reason records the kind of input change that forced a node back into the
//! restyle set, so the runtime can reason about, log and prioritise pending work
//! deterministically.

/// The typed reason a node's computed style was invalidated (§4.3).
///
/// Every entry in a [`crate::invalidation::StyleInvalidation`] set carries one of
/// these so the cause of a restyle is explicit rather than implied by a bare
/// dirty flag. `Copy + Eq + Hash` so it can be stored in maps/sets cheaply and
/// compared without allocation.
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

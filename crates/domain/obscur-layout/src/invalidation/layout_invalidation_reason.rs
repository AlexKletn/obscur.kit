//! The typed reason a layout context must be recomputed.

/// A typed classification of *why* a layout context must be recomputed
/// (spec §4.3).
///
/// §4.3 requires that changes be classified into typed reasons rather than
/// tracked through scattered ad-hoc dirty flags. Each variant identifies the
/// kind of change that reached a context.
///
/// # The variant itself is still write-only
///
/// The [`LayoutInvalidation`](super::LayoutInvalidation) a frame fills **is** read
/// — its node set drives the incremental measure pass (see
/// `LayoutCoordinator::layout_invalidated`) — but that pass branches on *which
/// nodes* are invalidated, never on *why*: it treats every reason below
/// identically and conservatively. So these variants are, today, a classification
/// the runtime records and no consumer discriminates on.
///
/// Reading them is what would let propagation follow §4.3's finer rules (upward
/// only on geometry change, downward only on constraint change) rather than the
/// whole-ancestor-chain closure the measure pass currently takes; that needs the
/// per-context [`LayoutDependency`](crate::context::LayoutDependency) wiring
/// nothing populates yet.
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
    /// A caller asked for this node's layout to be recomputed without naming a
    /// finer cause.
    ///
    /// The other variants each state *why* the geometry went stale, which lets a
    /// future incremental pass decide how far to propagate. This one states only
    /// that it did. It exists because a caller outside layout genuinely may not
    /// know more — the runtime's `InvalidateLayout` operation is exactly that
    /// statement — and mapping such a request onto one of the specific causes
    /// would put a claim in its mouth that it never made.
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

//! [`UiOperation`]: a single normalized UI mutation intent (spec §3.2).

use obscur_identity::NodeId;

use crate::invalidation::{Invalidation, InvalidationReason};

/// A single normalized UI operation: a typed intent that the frame pipeline
/// interprets against the domains (tree, style, layout).
///
/// Operations are *normalized*: a mutation coming from application code is
/// reduced to one of these variants before it enters a [`UiTransaction`], so
/// the runtime never dispatches on strings and always reasons over a closed set
/// of intents. This is only the *intent* — nothing is applied when a
/// `UiOperation` is created. The concrete effect happens later in the frame
/// pipeline once the transaction has been validated and committed.
///
/// # What an operation actually does
///
/// [`invalidation`](Self::invalidation) is where an intent becomes an effect: it
/// states the change as an [`Invalidation`], which the
/// [`InvalidationRouter`](crate::invalidation::InvalidationRouter) turns into the
/// set of pipeline phases a frame must run. That is the whole of it — an
/// operation *classifies work*, it does not describe an edit. The edit itself
/// comes from reconciling the desired [`View`](obscur_view::view::View); an
/// operation says which phases have to look at the result.
///
/// This is the Phase-1 set; more operation kinds are added in later phases.
///
/// [`UiTransaction`]: crate::transaction::UiTransaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiOperation {
    /// The root desired output changed and the whole tree must reconcile.
    ///
    /// This targets no specific node — it applies to the tree as a whole.
    ReplaceRoot,
    /// A component subtree rooted at `node` must be re-rendered.
    Rerender {
        /// The node at the root of the subtree to re-render.
        node: NodeId,
    },
    /// A node's style must be recomputed.
    InvalidateStyle {
        /// The node whose style is now stale.
        node: NodeId,
    },
    /// A node's layout must be recomputed.
    InvalidateLayout {
        /// The node whose layout is now stale.
        node: NodeId,
    },
}

impl UiOperation {
    /// States this operation as the [`Invalidation`] it causes — the runtime-level
    /// change a frame must classify (spec §3.4 phase 5, §4.3).
    ///
    /// This is the seam that gives the operation vocabulary an effect. Route the
    /// result through an
    /// [`InvalidationRouter`](crate::invalidation::InvalidationRouter) and its
    /// phase flags say what the frame recalculates.
    ///
    /// The mapping is total and each arm says what the operation *claims*:
    ///
    /// * [`ReplaceRoot`](Self::ReplaceRoot) — a **global**
    ///   [`TreeChanged`](InvalidationReason::TreeChanged). It names no node
    ///   because it targets none: the whole tree reconciles.
    /// * [`Rerender`](Self::Rerender) — `TreeChanged` **at that node**. A
    ///   component's output can change its subtree's structure, so this is the
    ///   same cause, narrowed.
    /// * [`InvalidateStyle`](Self::InvalidateStyle) —
    ///   [`StyleChanged`](InvalidationReason::StyleChanged): the node restyles,
    ///   and layout follows because a style can feed geometry.
    /// * [`InvalidateLayout`](Self::InvalidateLayout) —
    ///   [`LayoutChanged`](InvalidationReason::LayoutChanged): geometry only. It
    ///   deliberately does **not** dirty the style phase, which is the entire
    ///   difference from `InvalidateStyle` and the reason the two are separate
    ///   operations.
    #[must_use]
    pub fn invalidation(self) -> Invalidation {
        match self {
            Self::ReplaceRoot => Invalidation::global(InvalidationReason::TreeChanged),
            Self::Rerender { node } => {
                Invalidation::new(Some(node), InvalidationReason::TreeChanged)
            }
            Self::InvalidateStyle { node } => {
                Invalidation::new(Some(node), InvalidationReason::StyleChanged)
            }
            Self::InvalidateLayout { node } => {
                Invalidation::new(Some(node), InvalidationReason::LayoutChanged)
            }
        }
    }

    /// Returns the node this operation targets, or `None` for
    /// [`UiOperation::ReplaceRoot`], which applies to the whole tree rather than
    /// a single node.
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        match self {
            Self::ReplaceRoot => None,
            Self::Rerender { node }
            | Self::InvalidateStyle { node }
            | Self::InvalidateLayout { node } => Some(*node),
        }
    }

    /// Returns `true` if this operation replaces the root (a whole-tree reconcile).
    #[must_use]
    pub const fn is_root_replacement(&self) -> bool {
        matches!(self, Self::ReplaceRoot)
    }

    /// Returns `true` if this operation requests a component subtree re-render.
    #[must_use]
    pub const fn is_rerender(&self) -> bool {
        matches!(self, Self::Rerender { .. })
    }

    /// Returns `true` if this operation invalidates a node's style.
    #[must_use]
    pub const fn is_style_invalidation(&self) -> bool {
        matches!(self, Self::InvalidateStyle { .. })
    }

    /// Returns `true` if this operation invalidates a node's layout.
    #[must_use]
    pub const fn is_layout_invalidation(&self) -> bool {
        matches!(self, Self::InvalidateLayout { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_replacement_has_no_target() {
        let op = UiOperation::ReplaceRoot;
        assert_eq!(op.target(), None);
        assert!(op.is_root_replacement());
        assert!(!op.is_rerender());
    }

    #[test]
    fn node_operations_report_their_target() {
        let node = NodeId::from_raw(7);
        for op in [
            UiOperation::Rerender { node },
            UiOperation::InvalidateStyle { node },
            UiOperation::InvalidateLayout { node },
        ] {
            assert_eq!(op.target(), Some(node));
            assert!(!op.is_root_replacement());
        }
    }

    #[test]
    fn predicates_are_mutually_exclusive() {
        let node = NodeId::from_raw(1);
        assert!(UiOperation::Rerender { node }.is_rerender());
        assert!(UiOperation::InvalidateStyle { node }.is_style_invalidation());
        assert!(UiOperation::InvalidateLayout { node }.is_layout_invalidation());
        assert!(!UiOperation::Rerender { node }.is_style_invalidation());
    }
}

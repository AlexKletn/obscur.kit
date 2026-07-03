//! [`UiOperation`]: a single normalized UI mutation intent (spec §3.2).

use obscur_identity::NodeId;

/// A single normalized UI operation: a typed intent that the frame pipeline
/// interprets against the domains (tree, style, layout).
///
/// Operations are *normalized*: a mutation coming from application code is
/// reduced to one of these variants before it enters a [`UiTransaction`], so
/// the runtime never dispatches on strings and always reasons over a closed set
/// of intents. This is only the *intent* — nothing is applied when a
/// `UiOperation` is created. The concrete effect (reconciliation, style
/// recomputation, layout recomputation) happens later in the frame pipeline
/// once the transaction has been validated and committed.
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

//! A single portal: content whose logical position stays put but paints elsewhere.

use obscur_identity::NodeId;

use crate::node::{NodeFlags, VisualParent};
use crate::tree::RetainedTree;

use super::PortalTarget;

/// Represents one portal (spec §4.14).
///
/// A portal changes where a node is *painted* — its visual parent / stacking
/// position — while **preserving its logical parent**, and therefore its style
/// and theme scope. It pairs a **source** node (whose logical position is
/// unchanged) with a **target** ([`PortalTarget`]) naming the visual
/// destination host.
///
/// # Invariant (spec §4.14)
///
/// Applying a portal moves **only** the source node's visual parent to the
/// target host; its logical parent is left exactly as it was. Because logical
/// parentage defines style/theme scope, the portalled node keeps inheriting from
/// its original place in the tree even though it paints under the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Portal {
    /// The source node whose logical position stays put.
    source: NodeId,
    /// The visual destination the source is painted into.
    target: PortalTarget,
}

impl Portal {
    /// Creates a portal from `source` to the visual destination `target`.
    #[must_use]
    pub const fn new(source: NodeId, target: PortalTarget) -> Self {
        Self { source, target }
    }

    /// Returns the source node whose logical position is preserved.
    #[must_use]
    pub const fn source(self) -> NodeId {
        self.source
    }

    /// Returns the visual destination this portal renders into.
    #[must_use]
    pub const fn target(self) -> PortalTarget {
        self.target
    }

    /// Enacts the portal on `tree`.
    ///
    /// This sets the **source node's visual parent** to the target's host
    /// ([`VisualParent::of`]), **leaves its logical parent unchanged**
    /// (preserving style/theme scope, spec §4.14), and flags the **host** node
    /// with [`NodeFlags::PORTAL_HOST`].
    ///
    /// Returns `true` when both the source and the host are live and the portal
    /// was applied; returns `false` (a no-op) otherwise. It is defensive about
    /// stale ids: if either endpoint does not resolve to a live node, nothing is
    /// mutated and `false` is returned, so the call is panic-free.
    pub fn apply(self, tree: &mut RetainedTree) -> bool {
        let host = self.target.host_node();
        // Both endpoints must be live before mutating anything.
        if !tree.contains(self.source) || !tree.contains(host) {
            return false;
        }
        // Move only the visual parent; the logical parent is untouched.
        let Some(source) = tree.get_mut(self.source) else {
            return false;
        };
        source.set_visual_parent(VisualParent::of(host));
        // Mark the destination as a portal host.
        let Some(host_node) = tree.get_mut(host) else {
            return false;
        };
        host_node.insert_flags(NodeFlags::PORTAL_HOST);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{LogicalParent, NodeKind};

    /// Builds `logical_parent -> source` and a separate `host`, then portals
    /// `source` into `host`. Returns `(source, host, logical_parent)`.
    fn setup(tree: &mut RetainedTree) -> (NodeId, NodeId, NodeId) {
        let logical = tree.alloc(NodeKind::Root);
        let source = tree.alloc(NodeKind::Root);
        let host = tree.alloc(NodeKind::Root);
        if let Some(n) = tree.get_mut(logical) {
            n.push_child(source);
        }
        if let Some(n) = tree.get_mut(source) {
            n.set_logical_parent(LogicalParent::of(logical));
            n.set_visual_parent(VisualParent::of(logical));
        }
        (source, host, logical)
    }

    #[test]
    fn apply_preserves_logical_moves_visual_and_flags_host() {
        let mut tree = RetainedTree::new();
        let (source, host, logical) = setup(&mut tree);

        let portal = Portal::new(source, PortalTarget::host(host));
        assert!(portal.apply(&mut tree));

        let src = tree.get(source).expect("source live");
        // Logical parent unchanged (style/theme scope preserved, §4.14).
        assert_eq!(src.logical_parent(), LogicalParent::of(logical));
        // Visual parent now points at the host.
        assert_eq!(src.visual_parent(), VisualParent::of(host));

        // Host is flagged as a portal host.
        let host_node = tree.get(host).expect("host live");
        assert!(host_node.has_flags(NodeFlags::PORTAL_HOST));
    }

    #[test]
    fn accessors_roundtrip() {
        let source = NodeId::from_raw(1);
        let target = PortalTarget::host(NodeId::from_raw(2));
        let portal = Portal::new(source, target);
        assert_eq!(portal.source(), source);
        assert_eq!(portal.target(), target);
    }

    #[test]
    fn stale_source_is_a_noop() {
        let mut tree = RetainedTree::new();
        let host = tree.alloc(NodeKind::Root);
        let ghost = NodeId::from_raw(u64::MAX);
        let portal = Portal::new(ghost, PortalTarget::host(host));
        assert!(!portal.apply(&mut tree));
        // Host must not have been flagged by a failed apply.
        assert!(!tree.get(host).expect("host live").has_flags(NodeFlags::PORTAL_HOST));
    }

    #[test]
    fn stale_host_is_a_noop() {
        let mut tree = RetainedTree::new();
        let source = tree.alloc(NodeKind::Root);
        let ghost = NodeId::from_raw(u64::MAX);
        let portal = Portal::new(source, PortalTarget::host(ghost));
        assert!(!portal.apply(&mut tree));
        // Source visual parent must be untouched (still root).
        assert!(tree.get(source).expect("source live").visual_parent().is_root());
    }
}

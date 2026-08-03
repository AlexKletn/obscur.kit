#![doc = include_str!("portal.doc/_module.md")]

use obscur_identity::NodeId;

use crate::node::{NodeFlags, VisualParent};
use crate::tree::RetainedTree;

use super::PortalTarget;

#[doc = include_str!("portal.doc/Portal.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Portal {
    #[doc = include_str!("portal.doc/source.md")]
    source: NodeId,
    #[doc = include_str!("portal.doc/target.md")]
    target: PortalTarget,
}

impl Portal {
    #[doc = include_str!("portal.doc/new.md")]
    #[must_use]
    pub const fn new(source: NodeId, target: PortalTarget) -> Self {
        Self { source, target }
    }

    #[doc = include_str!("portal.doc/source_2.md")]
    #[must_use]
    pub const fn source(self) -> NodeId {
        self.source
    }

    #[doc = include_str!("portal.doc/target_2.md")]
    #[must_use]
    pub const fn target(self) -> PortalTarget {
        self.target
    }

    #[doc = include_str!("portal.doc/apply.md")]
    pub fn apply(self, tree: &mut RetainedTree) -> bool {
        let host = self.target.host_node();
        if !tree.contains(self.source) || !tree.contains(host) {
            return false;
        }
        let Some(source) = tree.get_mut(self.source) else {
            return false;
        };
        source.set_visual_parent(VisualParent::of(host));
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
        assert!(
            !tree
                .get(host)
                .expect("host live")
                .has_flags(NodeFlags::PORTAL_HOST)
        );
    }

    #[test]
    fn stale_host_is_a_noop() {
        let mut tree = RetainedTree::new();
        let source = tree.alloc(NodeKind::Root);
        let ghost = NodeId::from_raw(u64::MAX);
        let portal = Portal::new(source, PortalTarget::host(ghost));
        assert!(!portal.apply(&mut tree));
        // Source visual parent must be untouched (still root).
        assert!(
            tree.get(source)
                .expect("source live")
                .visual_parent()
                .is_root()
        );
    }
}

//! The visual destination a portal renders its content into.

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/portal/portal_target.doc/PortalTarget.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortalTarget {
    /// The host node that will visually parent the portalled content.
    host: NodeId,
}

impl PortalTarget {
    /// Creates a portal target rendering into the given `host` node.
    #[must_use]
    pub const fn new(host: NodeId) -> Self {
        Self { host }
    }

    #[doc = include_str!("../../docs/api/portal/portal_target.doc/host_2.md")]
    #[must_use]
    pub const fn host(host: NodeId) -> Self {
        Self::new(host)
    }

    /// Returns the host node this target renders its content into.
    #[must_use]
    pub const fn host_node(self) -> NodeId {
        self.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_host_are_equivalent() {
        let id = NodeId::from_raw(7);
        assert_eq!(PortalTarget::new(id), PortalTarget::host(id));
    }

    #[test]
    fn exposes_host_node() {
        let id = NodeId::from_raw(42);
        let target = PortalTarget::new(id);
        assert_eq!(target.host_node(), id);
    }

    #[test]
    fn is_copy() {
        let target = PortalTarget::new(NodeId::from_raw(1));
        let copy = target;
        // Both usable: `target` was not moved by the assignment above.
        assert_eq!(target, copy);
    }
}

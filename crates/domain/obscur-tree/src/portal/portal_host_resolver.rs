use obscur_identity::NodeId;

use crate::tree::RetainedTree;

/// Host port for mapping a local portal target name to a retained node.
pub trait PortalHostResolver {
    /// Returns the visual host in this tree. The runtime rejects stale hosts and projections that form a cycle.
    fn resolve(&self, tree: &RetainedTree, source: NodeId, target: &str) -> Option<NodeId>;
}

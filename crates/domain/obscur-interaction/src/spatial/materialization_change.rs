//! Membership changes in an application-managed spatial scene.

use obscur_identity::NodeId;

/// The difference between two materialized sets, with IDs in ascending order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterializationChange {
    /// Objects to materialize.
    pub entered: Vec<NodeId>,
    /// Objects to release.
    pub exited: Vec<NodeId>,
}

impl MaterializationChange {
    /// Returns whether the region update left membership unchanged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entered.is_empty() && self.exited.is_empty()
    }
}

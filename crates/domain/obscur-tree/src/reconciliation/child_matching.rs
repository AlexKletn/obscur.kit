//! `ChildMatching`: the pure output of matching one parent's children.

use obscur_identity::NodeId;

use super::IdentityMatch;

#[doc = include_str!("../../docs/api/reconciliation/child_matching.doc/ChildMatching.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChildMatching {
    matches: Vec<IdentityMatch>,
    removed: Vec<NodeId>,
}

impl ChildMatching {
    /// Builds a matching from the per-desired-child decisions and the list of
    /// unmatched existing children to remove.
    #[must_use]
    pub fn new(matches: Vec<IdentityMatch>, removed: Vec<NodeId>) -> Self {
        Self { matches, removed }
    }

    /// Returns the per-desired-child decisions, aligned with the desired
    /// children in order.
    #[must_use]
    pub fn matches(&self) -> &[IdentityMatch] {
        &self.matches
    }

    /// Returns the existing children left unmatched, which must be removed
    /// together with their subtrees.
    #[must_use]
    pub fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    /// Consumes this matching, returning its `(matches, removed)` parts.
    #[must_use]
    pub fn into_parts(self) -> (Vec<IdentityMatch>, Vec<NodeId>) {
        (self.matches, self.removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_expose_parts() {
        let matches = vec![
            IdentityMatch::Create,
            IdentityMatch::Reuse(NodeId::from_raw(1)),
        ];
        let removed = vec![NodeId::from_raw(2)];
        let matching = ChildMatching::new(matches.clone(), removed.clone());
        assert_eq!(matching.matches(), matches.as_slice());
        assert_eq!(matching.removed(), removed.as_slice());
        assert_eq!(matching.into_parts(), (matches, removed));
    }

    #[test]
    fn default_is_empty() {
        let matching = ChildMatching::default();
        assert!(matching.matches().is_empty());
        assert!(matching.removed().is_empty());
    }
}

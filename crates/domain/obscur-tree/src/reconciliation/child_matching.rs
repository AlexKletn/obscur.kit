#![doc = include_str!("child_matching.doc/_module.md")]

use obscur_identity::NodeId;

use super::IdentityMatch;

#[doc = include_str!("child_matching.doc/ChildMatching.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChildMatching {
    matches: Vec<IdentityMatch>,
    removed: Vec<NodeId>,
}

impl ChildMatching {
    #[doc = include_str!("child_matching.doc/new.md")]
    #[must_use]
    pub fn new(matches: Vec<IdentityMatch>, removed: Vec<NodeId>) -> Self {
        Self { matches, removed }
    }

    #[doc = include_str!("child_matching.doc/matches.md")]
    #[must_use]
    pub fn matches(&self) -> &[IdentityMatch] {
        &self.matches
    }

    #[doc = include_str!("child_matching.doc/removed.md")]
    #[must_use]
    pub fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    #[doc = include_str!("child_matching.doc/into_parts.md")]
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

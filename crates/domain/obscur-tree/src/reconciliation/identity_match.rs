#![doc = include_str!("identity_match.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("identity_match.doc/IdentityMatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityMatch {
    #[doc = include_str!("identity_match.doc/Reuse.md")]
    Reuse(NodeId),
    #[doc = include_str!("identity_match.doc/Create.md")]
    Create,
}

impl IdentityMatch {
    #[doc = include_str!("identity_match.doc/reused.md")]
    #[must_use]
    pub const fn reused(self) -> Option<NodeId> {
        match self {
            Self::Reuse(id) => Some(id),
            Self::Create => None,
        }
    }

    #[doc = include_str!("identity_match.doc/is_reuse.md")]
    #[must_use]
    pub const fn is_reuse(self) -> bool {
        matches!(self, Self::Reuse(_))
    }

    #[doc = include_str!("identity_match.doc/is_create.md")]
    #[must_use]
    pub const fn is_create(self) -> bool {
        matches!(self, Self::Create)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_exposes_id() {
        let id = NodeId::from_raw(7);
        let m = IdentityMatch::Reuse(id);
        assert!(m.is_reuse());
        assert!(!m.is_create());
        assert_eq!(m.reused(), Some(id));
    }

    #[test]
    fn create_has_no_id() {
        let m = IdentityMatch::Create;
        assert!(m.is_create());
        assert!(!m.is_reuse());
        assert_eq!(m.reused(), None);
    }
}

#![doc = include_str!("caret_affinity.doc/_module.md")]

#[doc = include_str!("caret_affinity.doc/CaretAffinity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum CaretAffinity {
    #[doc = include_str!("caret_affinity.doc/Upstream.md")]
    Upstream,
    #[doc = include_str!("caret_affinity.doc/Downstream.md")]
    #[default]
    Downstream,
}

impl CaretAffinity {
    #[doc = include_str!("caret_affinity.doc/is_upstream.md")]
    #[must_use]
    pub const fn is_upstream(self) -> bool {
        matches!(self, Self::Upstream)
    }

    #[doc = include_str!("caret_affinity.doc/is_downstream.md")]
    #[must_use]
    pub const fn is_downstream(self) -> bool {
        matches!(self, Self::Downstream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_downstream() {
        assert_eq!(CaretAffinity::default(), CaretAffinity::Downstream);
    }

    #[test]
    fn predicates() {
        assert!(CaretAffinity::Upstream.is_upstream());
        assert!(!CaretAffinity::Upstream.is_downstream());
        assert!(CaretAffinity::Downstream.is_downstream());
        assert!(!CaretAffinity::Downstream.is_upstream());
    }
}

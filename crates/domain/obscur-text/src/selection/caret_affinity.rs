//! Which side of a soft line-wrap boundary a caret binds to (spec §5.11).

#[doc = include_str!("../../docs/api/selection/caret_affinity.doc/CaretAffinity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum CaretAffinity {
    /// The offset binds to the trailing edge of the preceding line (end of the
    /// earlier line at a wrap boundary).
    Upstream,
    /// The offset binds to the leading edge of the following line (start of the
    /// later line at a wrap boundary). The default.
    #[default]
    Downstream,
}

impl CaretAffinity {
    /// Returns whether this affinity is [`Upstream`](Self::Upstream).
    #[must_use]
    pub const fn is_upstream(self) -> bool {
        matches!(self, Self::Upstream)
    }

    /// Returns whether this affinity is [`Downstream`](Self::Downstream).
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

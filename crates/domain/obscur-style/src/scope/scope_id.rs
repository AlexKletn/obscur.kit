#![doc = include_str!("scope_id.doc/_module.md")]

#[doc = include_str!("scope_id.doc/ScopeId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(u32);

impl ScopeId {
    #[doc = include_str!("scope_id.doc/GLOBAL.md")]
    pub const GLOBAL: ScopeId = ScopeId(0);

    #[doc = include_str!("scope_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[doc = include_str!("scope_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[doc = include_str!("scope_id.doc/is_global.md")]
    #[must_use]
    pub const fn is_global(self) -> bool {
        self.0 == Self::GLOBAL.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_raw() {
        assert_eq!(ScopeId::from_raw(42).raw(), 42);
    }

    #[test]
    fn global_is_reserved_zero() {
        assert!(ScopeId::GLOBAL.is_global());
        assert!(!ScopeId::from_raw(1).is_global());
    }

    #[test]
    fn is_hashable_and_ord() {
        use std::collections::BTreeSet;
        let mut set = BTreeSet::new();
        set.insert(ScopeId::from_raw(2));
        set.insert(ScopeId::from_raw(1));
        set.insert(ScopeId::from_raw(1));
        assert_eq!(set.len(), 2);
        assert_eq!(set.iter().next(), Some(&ScopeId::from_raw(1)));
    }
}

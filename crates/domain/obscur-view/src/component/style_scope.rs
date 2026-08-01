#![doc = include_str!("style_scope.doc/_module.md")]

#[doc = include_str!("style_scope.doc/StyleScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StyleScope {
    #[doc = include_str!("style_scope.doc/Scoped.md")]
    #[default]
    Scoped,
    #[doc = include_str!("style_scope.doc/Open.md")]
    Open,
}

impl StyleScope {
    #[doc = include_str!("style_scope.doc/is_open.md")]
    #[must_use]
    pub const fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_is_the_default() {
        // §4.15's "scoped by default" is the type's default, not a convention a
        // caller has to remember.
        assert_eq!(StyleScope::default(), StyleScope::Scoped);
        assert!(!StyleScope::default().is_open());
        assert!(StyleScope::Open.is_open());
    }
}

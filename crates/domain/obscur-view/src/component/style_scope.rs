//! Whether a component's rendered output is scoped away from the document's
//! stylesheets or open to them: [`StyleScope`] (spec §4.15).

#[doc = include_str!("../../docs/api/component/style_scope.doc/StyleScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StyleScope {
    /// The default: ordinary selectors from outside do not reach the
    /// component's rendered elements; only exported style parts cross.
    #[default]
    Scoped,
    /// The component's rendered elements are styled by the document's sheets
    /// exactly as authored elements are.
    Open,
}

impl StyleScope {
    /// Returns `true` when the document's ordinary selectors reach this
    /// component's rendered elements.
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

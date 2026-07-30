//! Whether a component's rendered output is scoped away from the document's
//! stylesheets or open to them: [`StyleScope`] (spec §4.15).

/// Whether the document's ordinary selectors reach the elements a component
/// renders (spec §4.15).
///
/// §4.15 lists both halves of the rule and the implementation owes both:
///
/// * "*component stylesheets are scoped by default*" — [`StyleScope::Scoped`],
///   the default. An outside sheet's ordinary selectors stop at the component
///   host; the only door through is an exported style part
///   (`Component::part(name)`), which is why "*internal classes do not need
///   global uniqueness*" holds. A component's internals are its own business.
/// * "*explicitly open/global scopes are possible*" — [`StyleScope::Open`]. The
///   component says, in its own contract, that its output is part of the
///   document for styling purposes: `.counter` outside matches `.counter`
///   inside, a descendant selector climbs from an authored ancestor into the
///   component's output, and no export is needed.
///
/// # Why this is a declaration and not a host-side switch
///
/// The scope belongs to the *component*, not to the application that uses it: a
/// component that hides its internals is making a compatibility promise about
/// them, and an application must not be able to break that promise from the
/// outside by reaching in. So the decision is authored where the promise is —
/// the contract — and the runtime reads it once, at mount.
///
/// # What used to happen
///
/// Only the closing half existed. Every mounted component's output was scoped
/// unconditionally, there is no way to attach a component-scoped stylesheet yet
/// (the [`ComponentModule`](../../../obscur_style/cascade/enum.CascadeOrigin.html)
/// origin has no producer), and so a component's rendered elements could be
/// styled by nothing at all except a `::part()` export — the external consumer
/// audit's finding 2-1, "a component-rendered subtree is matched by no
/// selector". Scoping was doing exactly what it was written to do; the missing
/// piece was the author's ability to say "not for this one".
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

//! A single simple selector: [`SimpleSelector`] and its attribute forms.

use super::pseudo_class_selector::PseudoClassSelector;
use super::selector_specificity::SelectorSpecificity;

/// A single simple selector — the atomic unit of a compound selector.
///
/// This mirrors the parsed selector surface produced by `obscur-css`; a later
/// sub-step lowers a parsed compound into a sequence of these. Names are stored
/// as owned strings rather than interned symbols so that matching against tree
/// nodes (which intern in a *different* table than CSS) is a plain string
/// comparison — see the crate's interner-coherence note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    /// A type/element selector such as `div`; stores the local name.
    Type(Box<str>),
    /// The universal selector `*`, matching every element.
    Universal,
    /// A class selector such as `.button`; stores the class name (no `.`).
    Class(Box<str>),
    /// An id selector such as `#main`; stores the id (no `#`).
    Id(Box<str>),
    /// An attribute selector such as `[data-x]` or `[type="text"]`.
    Attribute(AttributeSelector),
    /// A pseudo-class such as `:hover` or `:nth-child(2n+1)`.
    PseudoClass(PseudoClassSelector),
    /// A pseudo-element such as `::before`; stores the name (no `::`).
    ///
    /// Never matched by the ordinary single-element matcher: a pseudo-element
    /// selects a *generated box* (§4.16 `::before`/`::after`), not the
    /// originating element, so
    /// [`SelectorMatcher`](super::selector_matcher::SelectorMatcher) fails
    /// closed here rather than styling the originating element by mistake. The
    /// runtime resolves generated-content rules through a dedicated
    /// pseudo-element path (`matches_complex_for_pseudo`) that anchors the rule
    /// on the originating element while keeping its declarations off that
    /// element's own box.
    PseudoElement(Box<str>),
    /// An exported style part such as `::part(label)`; stores the part name
    /// (no `::part(` / `)`), §2.12 / §4.15.
    ///
    /// Like [`PseudoElement`](Self::PseudoElement) it is inert in the ordinary
    /// single-element matcher: `Component::part(name)` matches an element in a
    /// component's *scoped* render output that the component exports under
    /// `name`, and deciding that needs the component-host boundary — tree
    /// context the pure matcher does not have. The runtime's style-matching
    /// coordinator matches it against the exported part while the rest of the
    /// compound matches the host.
    Part(Box<str>),
}

impl SimpleSelector {
    /// Returns the part name if this is a [`Part`](Self::Part) selector.
    #[must_use]
    pub fn part_name(&self) -> Option<&str> {
        match self {
            Self::Part(name) => Some(name),
            _ => None,
        }
    }

    /// Returns the pseudo-element name if this is a
    /// [`PseudoElement`](Self::PseudoElement) selector.
    #[must_use]
    pub fn pseudo_element_name(&self) -> Option<&str> {
        match self {
            Self::PseudoElement(name) => Some(name),
            _ => None,
        }
    }
}

impl SimpleSelector {
    /// Returns the specificity this simple selector contributes.
    ///
    /// Per CSS: id → `(1, 0, 0)`; class, attribute, and pseudo-class →
    /// `(0, 1, 0)`; type and pseudo-element → `(0, 0, 1)`; the universal
    /// selector contributes nothing.
    ///
    /// Pseudo-classes delegate to
    /// [`PseudoClassSelector::specificity_contribution`], because the four
    /// selectors-4 §15 "evaluation context" forms — `:is()`, `:not()`,
    /// `:where()`, `:has()` — do **not** contribute a flat `(0, 1, 0)`.
    #[must_use]
    pub fn specificity_contribution(&self) -> SelectorSpecificity {
        match self {
            Self::Id(_) => SelectorSpecificity::new(1, 0, 0),
            Self::PseudoClass(pseudo) => pseudo.specificity_contribution(),
            Self::Class(_) | Self::Attribute(_) => SelectorSpecificity::new(0, 1, 0),
            // A pseudo-element (`::before`) and the `::part()` pseudo-element
            // both contribute like a pseudo-element: (0, 0, 1).
            Self::Type(_) | Self::PseudoElement(_) | Self::Part(_) => {
                SelectorSpecificity::new(0, 0, 1)
            }
            Self::Universal => SelectorSpecificity::ZERO,
        }
    }
}

/// An attribute selector: a required attribute name plus an optional value test.
///
/// A [`None`] matcher is a bare presence test (`[disabled]`); otherwise the
/// [`AttrMatch`] describes how the attribute's value must relate to a compared
/// string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    /// The attribute name that must be present.
    pub name: Box<str>,
    /// The value test, or [`None`] for a bare presence test.
    pub matcher: Option<AttrMatch>,
}

/// The value-comparison operator of an attribute selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrMatch {
    /// `[a=b]` — the value equals the operand exactly.
    Exact(Box<str>),
    /// `[a~=b]` — the value is a whitespace-separated list containing the operand.
    Includes(Box<str>),
    /// `[a|=b]` — the value equals the operand or begins with `operand-`.
    DashMatch(Box<str>),
    /// `[a^=b]` — the value begins with the operand.
    Prefix(Box<str>),
    /// `[a$=b]` — the value ends with the operand.
    Suffix(Box<str>),
    /// `[a*=b]` — the value contains the operand as a substring.
    Substring(Box<str>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_contributes_a() {
        let s = SimpleSelector::Id("main".into());
        assert_eq!(
            s.specificity_contribution(),
            SelectorSpecificity::new(1, 0, 0)
        );
    }

    #[test]
    fn class_attribute_pseudo_class_contribute_b() {
        assert_eq!(
            SimpleSelector::Class("x".into()).specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
        assert_eq!(
            SimpleSelector::Attribute(AttributeSelector {
                name: "x".into(),
                matcher: None
            })
            .specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
        assert_eq!(
            SimpleSelector::PseudoClass(PseudoClassSelector::Hover).specificity_contribution(),
            SelectorSpecificity::new(0, 1, 0)
        );
    }

    #[test]
    fn type_and_pseudo_element_contribute_c() {
        assert_eq!(
            SimpleSelector::Type("div".into()).specificity_contribution(),
            SelectorSpecificity::new(0, 0, 1)
        );
        assert_eq!(
            SimpleSelector::PseudoElement("before".into()).specificity_contribution(),
            SelectorSpecificity::new(0, 0, 1)
        );
    }

    #[test]
    fn universal_contributes_zero() {
        assert_eq!(
            SimpleSelector::Universal.specificity_contribution(),
            SelectorSpecificity::ZERO
        );
    }
}

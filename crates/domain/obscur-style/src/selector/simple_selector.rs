//! A single simple selector: [`SimpleSelector`] and its attribute forms.

use super::pseudo_class_selector::PseudoClassSelector;
use super::selector_specificity::SelectorSpecificity;

use super::AttributeSelector;

#[doc = include_str!("../../docs/api/selector/simple_selector.doc/SimpleSelector.md")]
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
    #[doc = include_str!("../../docs/api/selector/simple_selector.doc/PseudoElement.md")]
    PseudoElement(Box<str>),
    #[doc = include_str!("../../docs/api/selector/simple_selector.doc/Part.md")]
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
    #[doc = include_str!("../../docs/api/selector/simple_selector.doc/specificity_contribution.md")]
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

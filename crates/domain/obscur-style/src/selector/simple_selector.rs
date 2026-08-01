#![doc = include_str!("simple_selector.doc/_module.md")]

use super::pseudo_class_selector::PseudoClassSelector;
use super::selector_specificity::SelectorSpecificity;

#[doc = include_str!("simple_selector.doc/SimpleSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    #[doc = include_str!("simple_selector.doc/Type.md")]
    Type(Box<str>),
    #[doc = include_str!("simple_selector.doc/Universal.md")]
    Universal,
    #[doc = include_str!("simple_selector.doc/Class.md")]
    Class(Box<str>),
    #[doc = include_str!("simple_selector.doc/Id.md")]
    Id(Box<str>),
    #[doc = include_str!("simple_selector.doc/Attribute.md")]
    Attribute(AttributeSelector),
    #[doc = include_str!("simple_selector.doc/PseudoClass.md")]
    PseudoClass(PseudoClassSelector),
    #[doc = include_str!("simple_selector.doc/PseudoElement.md")]
    PseudoElement(Box<str>),
    #[doc = include_str!("simple_selector.doc/Part.md")]
    Part(Box<str>),
}

impl SimpleSelector {
    #[doc = include_str!("simple_selector.doc/part_name.md")]
    #[must_use]
    pub fn part_name(&self) -> Option<&str> {
        match self {
            Self::Part(name) => Some(name),
            _ => None,
        }
    }

    #[doc = include_str!("simple_selector.doc/pseudo_element_name.md")]
    #[must_use]
    pub fn pseudo_element_name(&self) -> Option<&str> {
        match self {
            Self::PseudoElement(name) => Some(name),
            _ => None,
        }
    }
}

impl SimpleSelector {
    #[doc = include_str!("simple_selector.doc/specificity_contribution.md")]
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

#[doc = include_str!("simple_selector.doc/AttributeSelector.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
    #[doc = include_str!("simple_selector.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("simple_selector.doc/matcher.md")]
    pub matcher: Option<AttrMatch>,
}

#[doc = include_str!("simple_selector.doc/AttrMatch.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrMatch {
    #[doc = include_str!("simple_selector.doc/Exact.md")]
    Exact(Box<str>),
    #[doc = include_str!("simple_selector.doc/Includes.md")]
    Includes(Box<str>),
    #[doc = include_str!("simple_selector.doc/DashMatch.md")]
    DashMatch(Box<str>),
    #[doc = include_str!("simple_selector.doc/Prefix.md")]
    Prefix(Box<str>),
    #[doc = include_str!("simple_selector.doc/Suffix.md")]
    Suffix(Box<str>),
    #[doc = include_str!("simple_selector.doc/Substring.md")]
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

//! A style rule selecting one class.

use obscur_style::matching::{BlockDeclaration, StyleRule};
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};
use obscur_style::selector::{
    ComplexSelector, CompoundSelector, PseudoClassSelector, SelectorList, SimpleSelector,
};

#[doc = include_str!("../../docs/api/fixture/class_rule.doc/ClassRule.md")]
#[derive(Debug, Clone)]
pub struct ClassRule {
    /// The class name the selector matches.
    class: Box<str>,
    /// The pseudo-class qualifying the match, if any.
    pseudo: Option<PseudoClassSelector>,
    /// The declarations the rule applies, in the order they were added.
    declarations: Vec<BlockDeclaration>,
}

impl ClassRule {
    /// A rule matching `.class` in any state.
    #[must_use]
    pub fn new(class: &str) -> Self {
        Self {
            class: class.into(),
            pseudo: None,
            declarations: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/fixture/class_rule.doc/hover.md")]
    #[must_use]
    pub fn hover(class: &str) -> Self {
        Self {
            pseudo: Some(PseudoClassSelector::Hover),
            ..Self::new(class)
        }
    }

    /// A rule matching `.class:focus`, the focus counterpart of [`hover`](Self::hover).
    #[must_use]
    pub fn focus(class: &str) -> Self {
        Self {
            pseudo: Some(PseudoClassSelector::Focus),
            ..Self::new(class)
        }
    }

    /// Adds `width` and `height`, both in `px`.
    #[must_use]
    pub fn sized(self, width: f32, height: f32) -> Self {
        self.declaring(vec![
            BlockDeclaration::normal(PropertyId::Width, PropertyValue::Length(Length::px(width))),
            BlockDeclaration::normal(
                PropertyId::Height,
                PropertyValue::Length(Length::px(height)),
            ),
        ])
    }

    /// Adds an opaque `background-color` from the three channels.
    #[must_use]
    pub fn background(self, red: f32, green: f32, blue: f32) -> Self {
        self.declaring(vec![BlockDeclaration::normal(
            PropertyId::BackgroundColor,
            PropertyValue::Color(StyleColor::rgba(red, green, blue, 1.0)),
        )])
    }

    #[doc = include_str!("../../docs/api/fixture/class_rule.doc/scrollable.md")]
    #[must_use]
    pub fn scrollable(self) -> Self {
        // What `overflow: auto` compiles to: the shorthand is expanded into the
        // two axis longhands, and a fixture states what a sheet would.
        self.declaring(vec![
            BlockDeclaration::normal(
                PropertyId::OverflowX,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO),
            ),
            BlockDeclaration::normal(
                PropertyId::OverflowY,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO),
            ),
        ])
    }

    /// Appends declarations the named methods do not cover.
    #[must_use]
    pub fn declaring(mut self, declarations: Vec<BlockDeclaration>) -> Self {
        self.declarations.extend(declarations);
        self
    }

    /// Assembles the selector and the declarations into the rule.
    #[must_use]
    pub fn build(self) -> StyleRule {
        let mut compound = vec![SimpleSelector::Class(self.class)];
        if let Some(pseudo) = self.pseudo {
            compound.push(SimpleSelector::PseudoClass(pseudo));
        }
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(compound),
            Vec::new(),
        )]);
        StyleRule::new(selectors, self.declarations)
    }
}

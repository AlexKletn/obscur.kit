#![doc = include_str!("class_rule.doc/_module.md")]

use obscur_style::matching::{BlockDeclaration, StyleRule};
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor, layout_keyword};
use obscur_style::selector::{
    ComplexSelector, CompoundSelector, PseudoClassSelector, SelectorList, SimpleSelector,
};

#[doc = include_str!("class_rule.doc/ClassRule.md")]
#[derive(Debug, Clone)]
pub struct ClassRule {
    #[doc = include_str!("class_rule.doc/class.md")]
    class: Box<str>,
    #[doc = include_str!("class_rule.doc/pseudo.md")]
    pseudo: Option<PseudoClassSelector>,
    #[doc = include_str!("class_rule.doc/declarations.md")]
    declarations: Vec<BlockDeclaration>,
}

impl ClassRule {
    #[doc = include_str!("class_rule.doc/new.md")]
    #[must_use]
    pub fn new(class: &str) -> Self {
        Self {
            class: class.into(),
            pseudo: None,
            declarations: Vec::new(),
        }
    }

    #[doc = include_str!("class_rule.doc/hover.md")]
    #[must_use]
    pub fn hover(class: &str) -> Self {
        Self {
            pseudo: Some(PseudoClassSelector::Hover),
            ..Self::new(class)
        }
    }

    #[doc = include_str!("class_rule.doc/focus.md")]
    #[must_use]
    pub fn focus(class: &str) -> Self {
        Self {
            pseudo: Some(PseudoClassSelector::Focus),
            ..Self::new(class)
        }
    }

    #[doc = include_str!("class_rule.doc/sized.md")]
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

    #[doc = include_str!("class_rule.doc/background.md")]
    #[must_use]
    pub fn background(self, red: f32, green: f32, blue: f32) -> Self {
        self.declaring(vec![BlockDeclaration::normal(
            PropertyId::BackgroundColor,
            PropertyValue::Color(StyleColor::rgba(red, green, blue, 1.0)),
        )])
    }

    #[doc = include_str!("class_rule.doc/scrollable.md")]
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

    #[doc = include_str!("class_rule.doc/declaring.md")]
    #[must_use]
    pub fn declaring(mut self, declarations: Vec<BlockDeclaration>) -> Self {
        self.declarations.extend(declarations);
        self
    }

    #[doc = include_str!("class_rule.doc/build.md")]
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

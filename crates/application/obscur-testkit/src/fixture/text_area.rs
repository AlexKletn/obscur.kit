#![doc = include_str!("text_area.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_view::view::{
    CLASS, EVENT_INPUT, TYPE, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

use crate::fixture::{ClassRule, EditablePanel};

#[doc = include_str!("text_area.doc/TextArea.md")]
#[derive(Debug)]
pub struct TextArea;

impl TextArea {
    #[doc = include_str!("text_area.doc/LINE_HEIGHT.md")]
    pub const LINE_HEIGHT: f32 = 20.0;

    #[doc = include_str!("text_area.doc/SCROLLPORT_HEIGHT.md")]
    pub const SCROLLPORT_HEIGHT: f32 = 60.0;

    #[doc = include_str!("text_area.doc/view.md")]
    #[must_use]
    pub fn view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(710))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(711))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("area".into())))
                        .with_prop(ViewProperty::new(TYPE, ViewValue::from("textarea")))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    #[doc = include_str!("text_area.doc/sheet.md")]
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Self::sheet_with(Self::area_rule(Vec::new()))
    }

    #[doc = include_str!("text_area.doc/scrolling_sheet.md")]
    #[must_use]
    pub fn scrolling_sheet() -> Stylesheet {
        Self::sheet_with(Self::area_rule(vec![
            BlockDeclaration::normal(
                PropertyId::Height,
                PropertyValue::Length(Length::px(Self::SCROLLPORT_HEIGHT)),
            ),
            BlockDeclaration::normal(
                PropertyId::Overflow,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO),
            ),
        ]))
    }

    #[doc = include_str!("text_area.doc/sheet_with.md")]
    fn sheet_with(area: StyleRule) -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![ClassRule::new("panel").sized(400.0, 200.0).build(), area],
        )
    }

    #[doc = include_str!("text_area.doc/area_rule.md")]
    fn area_rule(extra: Vec<BlockDeclaration>) -> StyleRule {
        ClassRule::new("area")
            .declaring(vec![
                BlockDeclaration::normal(
                    PropertyId::Width,
                    PropertyValue::Length(Length::px(300.0)),
                ),
                BlockDeclaration::normal(
                    PropertyId::LineHeight,
                    PropertyValue::Length(Length::px(Self::LINE_HEIGHT)),
                ),
                BlockDeclaration::normal(
                    PropertyId::WhiteSpace,
                    PropertyValue::Keyword(layout_keyword::WHITE_SPACE_PRE_WRAP),
                ),
            ])
            .declaring(extra)
            .build()
    }
}

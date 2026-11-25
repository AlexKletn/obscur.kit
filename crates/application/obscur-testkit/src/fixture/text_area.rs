//! The multi-line editable: the one control where `Enter` is an edit rather than a
//! commit.

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_view::view::{
    CLASS, EVENT_INPUT, LOCAL_NAME, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

use crate::fixture::{ClassRule, EditablePanel};

#[doc = include_str!("../../docs/api/fixture/text_area.doc/TextArea.md")]
#[derive(Debug)]
pub struct TextArea;

impl TextArea {
    #[doc = include_str!("../../docs/api/fixture/text_area.doc/LINE_HEIGHT.md")]
    pub const LINE_HEIGHT: f32 = 20.0;

    /// The scrolling sheet's content-box height, in px — three
    /// [`LINE_HEIGHT`](Self::LINE_HEIGHT) lines exactly, with no padding or border, so
    /// a fourth typed line is the first one that must scroll.
    pub const SCROLLPORT_HEIGHT: f32 = 60.0;

    /// The textarea holding `text`, whose `\n`s are preserved as separate lines.
    #[must_use]
    pub fn view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(710))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(711))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("area".into())))
                        .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("textarea")))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// The non-scrolling sheet: a 400×200 `panel` over a 300 px-wide `area` with an
    /// exact [`LINE_HEIGHT`](Self::LINE_HEIGHT) and `white-space: pre-wrap`, and no
    /// height of its own — so the box grows with the content instead of scrolling.
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Self::sheet_with(Self::area_rule(Vec::new()))
    }

    /// The same sheet with the `area` fixed at [`SCROLLPORT_HEIGHT`](Self::SCROLLPORT_HEIGHT)
    /// and `overflow: auto`, making it a vertical scroll container.
    #[must_use]
    pub fn scrolling_sheet() -> Stylesheet {
        Self::sheet_with(Self::area_rule(vec![
            BlockDeclaration::normal(
                PropertyId::Height,
                PropertyValue::Length(Length::px(Self::SCROLLPORT_HEIGHT)),
            ),
            BlockDeclaration::normal(
                PropertyId::OverflowX,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO),
            ),
            BlockDeclaration::normal(
                PropertyId::OverflowY,
                PropertyValue::Keyword(layout_keyword::OVERFLOW_AUTO),
            ),
        ]))
    }

    /// The shared `panel` rule, over whichever `area` rule the caller built.
    fn sheet_with(area: StyleRule) -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![ClassRule::new("panel").sized(400.0, 200.0).build(), area],
        )
    }

    /// The `area` rule both sheets start from, plus whatever `extra` the scrolling one
    /// adds.
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

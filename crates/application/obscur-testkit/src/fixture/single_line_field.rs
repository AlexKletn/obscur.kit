//! The two shapes a *single-line* editable takes when its value does not fit: the
//! one that scrolls, and the one that wraps.

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_view::view::{
    CLASS, EVENT_INPUT, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use crate::fixture::{ClassRule, EditablePanel};

#[doc = include_str!("../../docs/api/fixture/single_line_field.doc/SingleLineField.md")]
#[derive(Debug)]
pub struct SingleLineField;

impl SingleLineField {
    /// The 100 px `nowrap`, `overflow: hidden` field, holding `text`.
    #[must_use]
    pub fn view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(600))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(601))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("field".into())))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Sheet for [`view`](Self::view): a 200×100 `panel` and a 100×20 single-line
    /// (`nowrap`, `overflow: hidden`) `field`. `white-space` inherits, so the field's
    /// text child shapes on one line.
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("panel").sized(200.0, 100.0).build(),
                ClassRule::new("field")
                    .sized(100.0, 20.0)
                    .declaring(vec![
                        BlockDeclaration::normal(
                            PropertyId::WhiteSpace,
                            PropertyValue::Keyword(layout_keyword::WHITE_SPACE_NOWRAP),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::OverflowX,
                            PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::OverflowY,
                            PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN),
                        ),
                    ])
                    .build(),
            ],
        )
    }

    /// The narrow wrapping field, holding `text` — the same shape as
    /// [`view`](Self::view) but with a `wrapfield` class, so a space-separated value
    /// wraps to several lines.
    #[must_use]
    pub fn wrapping_view(text: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(700))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(701))
                        .with_prop(ViewProperty::new(
                            CLASS,
                            ViewValue::Text("wrapfield".into()),
                        ))
                        .with_event(ViewEventBinding::new(EVENT_INPUT, EditablePanel::CMD_INPUT))
                        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                            text.to_string(),
                        )])),
                ])),
        ]))
    }

    /// Sheet for [`wrapping_view`](Self::wrapping_view): a 40 px-wide `wrapfield` with
    /// an exact 20 px `line-height` and `white-space: normal`.
    #[must_use]
    pub fn wrapping_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("panel").sized(200.0, 100.0).build(),
                ClassRule::new("wrapfield")
                    .declaring(vec![
                        BlockDeclaration::normal(
                            PropertyId::Width,
                            PropertyValue::Length(Length::px(40.0)),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::LineHeight,
                            PropertyValue::Length(Length::px(20.0)),
                        ),
                        BlockDeclaration::normal(
                            PropertyId::WhiteSpace,
                            PropertyValue::Keyword(layout_keyword::WHITE_SPACE_NORMAL),
                        ),
                    ])
                    .build(),
            ],
        )
    }
}

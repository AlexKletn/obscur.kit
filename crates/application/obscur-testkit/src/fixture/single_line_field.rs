#![doc = include_str!("single_line_field.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, layout_keyword};
use obscur_view::view::{
    CLASS, EVENT_INPUT, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use crate::fixture::{ClassRule, EditablePanel};

#[doc = include_str!("single_line_field.doc/SingleLineField.md")]
#[derive(Debug)]
pub struct SingleLineField;

impl SingleLineField {
    #[doc = include_str!("single_line_field.doc/view.md")]
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

    #[doc = include_str!("single_line_field.doc/sheet.md")]
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
                            PropertyId::Overflow,
                            PropertyValue::Keyword(layout_keyword::OVERFLOW_HIDDEN),
                        ),
                    ])
                    .build(),
            ],
        )
    }

    #[doc = include_str!("single_line_field.doc/wrapping_view.md")]
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

    #[doc = include_str!("single_line_field.doc/wrapping_sheet.md")]
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

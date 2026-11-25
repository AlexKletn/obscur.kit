//! The one-element view most runtime tests are written against.

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue};
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use crate::fixture::ClassRule;

#[doc = include_str!("../../docs/api/fixture/counter_panel.doc/CounterPanel.md")]
#[derive(Debug)]
pub struct CounterPanel;

impl CounterPanel {
    /// The lone `.counter` element.
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element()]))
    }

    #[doc = include_str!("../../docs/api/fixture/counter_panel.doc/nested_view.md")]
    #[must_use]
    pub fn nested_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            Self::element().with_children(ViewFragment::from_nodes(vec![ViewNode::element(
                ComponentId::from_raw(11),
            )])),
        ]))
    }

    /// `.counter { background-color: red; width: 100px; height: 50px }`.
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("counter")
                    .background(1.0, 0.0, 0.0)
                    .sized(100.0, 50.0)
                    .build(),
            ],
        )
    }

    #[doc = include_str!("../../docs/api/fixture/counter_panel.doc/height_only_sheet.md")]
    #[must_use]
    pub fn height_only_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("counter")
                    .background(1.0, 0.0, 0.0)
                    .declaring(vec![BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    )])
                    .build(),
            ],
        )
    }

    /// The `.counter` element node both views are built from.
    fn element() -> ViewNode {
        ViewNode::element(ComponentId::from_raw(10))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())))
    }
}

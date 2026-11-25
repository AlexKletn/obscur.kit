//! The box whose hover is either layout-affecting or paint-only.

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use crate::fixture::ClassRule;

#[doc = include_str!("../../docs/api/fixture/hover_box.doc/HoverBox.md")]
#[derive(Debug)]
pub struct HoverBox;

impl HoverBox {
    /// A single `.box` element.
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(10)]))
    }

    /// Three sibling `.box` elements, so a hover on one has clean siblings to be about.
    #[must_use]
    pub fn row_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(
            (0..3).map(|i| Self::element(10 + i)).collect(),
        ))
    }

    /// `.box { width: 100px; height: 50px }` and nothing else.
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(CascadeOrigin::GlobalApp, vec![Self::base_rule()])
    }

    /// `.box { width: 100px; height: 50px } .box:hover { width: 200px }`.
    #[must_use]
    pub fn layout_hover_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                Self::base_rule(),
                ClassRule::hover("box")
                    .declaring(vec![BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(200.0)),
                    )])
                    .build(),
            ],
        )
    }

    /// `.box { width: 100px; height: 50px } .box:hover { background-color: red }` — a
    /// hover that shifts only a fill (the wireframe kit's own hover model).
    #[must_use]
    pub fn paint_hover_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                Self::base_rule(),
                ClassRule::hover("box")
                    .declaring(vec![BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
                    )])
                    .build(),
            ],
        )
    }

    /// The selector list `.box`.
    #[must_use]
    pub fn selectors() -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("box".into())]),
            Vec::new(),
        )])
    }

    /// The unhovered `.box` rule both hover sheets open with.
    fn base_rule() -> obscur_style::matching::StyleRule {
        ClassRule::new("box").sized(100.0, 50.0).build()
    }

    /// One `.box` element under the given component id.
    fn element(component: u64) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(component))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
    }
}

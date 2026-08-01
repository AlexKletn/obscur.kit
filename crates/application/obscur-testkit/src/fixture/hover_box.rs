#![doc = include_str!("hover_box.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use crate::fixture::ClassRule;

#[doc = include_str!("hover_box.doc/HoverBox.md")]
#[derive(Debug)]
pub struct HoverBox;

impl HoverBox {
    #[doc = include_str!("hover_box.doc/view.md")]
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(10)]))
    }

    #[doc = include_str!("hover_box.doc/row_view.md")]
    #[must_use]
    pub fn row_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(
            (0..3).map(|i| Self::element(10 + i)).collect(),
        ))
    }

    #[doc = include_str!("hover_box.doc/sheet.md")]
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(CascadeOrigin::GlobalApp, vec![Self::base_rule()])
    }

    #[doc = include_str!("hover_box.doc/layout_hover_sheet.md")]
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

    #[doc = include_str!("hover_box.doc/paint_hover_sheet.md")]
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

    #[doc = include_str!("hover_box.doc/selectors.md")]
    #[must_use]
    pub fn selectors() -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("box".into())]),
            Vec::new(),
        )])
    }

    #[doc = include_str!("hover_box.doc/base_rule.md")]
    fn base_rule() -> obscur_style::matching::StyleRule {
        ClassRule::new("box").sized(100.0, 50.0).build()
    }

    #[doc = include_str!("hover_box.doc/element.md")]
    fn element(component: u64) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(component))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
    }
}

//! The element an image is loaded into.

use obscur_identity::ComponentId;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::{BlockDeclaration, Stylesheet};
use obscur_style::property::{PropertyId, PropertyValue};
use obscur_view::view::{CLASS, SRC, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use crate::fixture::ClassRule;

#[doc = include_str!("../../docs/api/fixture/image_panel.doc/ImagePanel.md")]
#[derive(Debug)]
pub struct ImagePanel;

impl ImagePanel {
    /// The asset url the sheet references, and the key it lands in the cache under.
    pub const URL: &'static str = "x";

    /// The lone `.img` element.
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into()))),
        ]))
    }

    /// `.img { background-image: url("x"); width: 40px; height: 30px }`.
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("img")
                    .declaring(vec![BlockDeclaration::normal(
                        PropertyId::BackgroundImage,
                        PropertyValue::BackgroundImageUrl(Self::URL.into()),
                    )])
                    .sized(40.0, 30.0)
                    .build(),
            ],
        )
    }

    #[doc = include_str!("../../docs/api/fixture/image_panel.doc/src_view.md")]
    #[must_use]
    pub fn src_view(src: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(11))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())))
                .with_prop(ViewProperty::new(SRC, ViewValue::Text(src.into()))),
        ]))
    }
}

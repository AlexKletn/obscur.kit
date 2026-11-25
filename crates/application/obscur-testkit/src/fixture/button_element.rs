//! The `button` a UA sheet and an author sheet argue over.

use obscur_identity::ComponentId;
use obscur_style::matching::{BlockDeclaration, StyleRule};
use obscur_style::property::{PropertyId, PropertyValue, StyleColor};
use obscur_style::selector::{
    ComplexSelector, CompoundSelector, PseudoClassSelector, SelectorList, SimpleSelector,
};
use obscur_view::view::{
    DISABLED, LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("../../docs/api/fixture/button_element.doc/ButtonElement.md")]
#[derive(Debug)]
pub struct ButtonElement;

impl ButtonElement {
    #[doc = include_str!("../../docs/api/fixture/button_element.doc/UA_FILL.md")]
    pub const UA_FILL: StyleColor =
        StyleColor::rgba(244.0 / 255.0, 242.0 / 255.0, 238.0 / 255.0, 1.0);

    /// The bare `button` element with its text child.
    #[must_use]
    pub fn view() -> View {
        Self::wrap(Self::element())
    }

    #[doc = include_str!("../../docs/api/fixture/button_element.doc/state_view.md")]
    #[must_use]
    pub fn state_view(disabled: bool) -> View {
        let mut node = Self::element();
        if disabled {
            node = node.with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true)));
        }
        Self::wrap(node)
    }

    /// `button { background-color: <color> }`.
    #[must_use]
    pub fn filled(color: StyleColor) -> StyleRule {
        StyleRule::new(Self::selectors(None), Self::background(color))
    }

    /// `button:disabled { background-color: <color> }`.
    #[must_use]
    pub fn disabled_filled(color: StyleColor) -> StyleRule {
        StyleRule::new(
            Self::selectors(Some(PseudoClassSelector::Disabled)),
            Self::background(color),
        )
    }

    /// The `button` element node both views are built from.
    fn element() -> ViewNode {
        ViewNode::element(ComponentId::from_raw(20))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("button".into()),
            ))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")]))
    }

    /// One element as a whole view.
    fn wrap(node: ViewNode) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![node]))
    }

    /// The `button` type selector, optionally compounded with a pseudo-class.
    fn selectors(pseudo: Option<PseudoClassSelector>) -> SelectorList {
        let mut compound = vec![SimpleSelector::Type("button".into())];
        if let Some(pseudo) = pseudo {
            compound.push(SimpleSelector::PseudoClass(pseudo));
        }
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(compound),
            Vec::new(),
        )])
    }

    /// A one-declaration block setting `background-color`.
    fn background(color: StyleColor) -> Vec<BlockDeclaration> {
        vec![BlockDeclaration::normal(
            PropertyId::BackgroundColor,
            PropertyValue::Color(color),
        )]
    }
}

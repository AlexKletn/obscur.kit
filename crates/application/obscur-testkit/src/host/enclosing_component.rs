//! The boundary *above* the one that failed.

mod no_props;
mod outer;
mod outer_factory;

use obscur_identity::ComponentId;
use obscur_view::component::{ComponentRegistration, ComponentVersion};
use obscur_view::view::{View, ViewFragment, ViewNode};
use outer_factory::OuterFactory;

#[doc = include_str!("../../docs/api/host/enclosing_component.doc/EnclosingComponent.md")]
#[derive(Debug)]
pub struct EnclosingComponent;

impl EnclosingComponent {
    /// The kind a view element names to mount this component.
    pub const ID: ComponentId = ComponentId::from_raw(0x0011_17E4);

    /// A registration for this component. It carries no configuration — the variation
    /// lives in the guarded component underneath it.
    #[must_use]
    pub fn registration() -> ComponentRegistration {
        ComponentRegistration::new(Box::new(OuterFactory), ComponentVersion::INITIAL)
    }

    /// A view with one `<Outer/>` element.
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(Self::ID)]))
    }
}

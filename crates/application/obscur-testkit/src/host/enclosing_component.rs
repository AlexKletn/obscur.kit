#![doc = include_str!("enclosing_component.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId, PropsError,
};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use crate::host::GuardedComponent;

#[doc = include_str!("enclosing_component.doc/EnclosingComponent.md")]
#[derive(Debug)]
pub struct EnclosingComponent;

impl EnclosingComponent {
    #[doc = include_str!("enclosing_component.doc/ID.md")]
    pub const ID: ComponentId = ComponentId::from_raw(0x0011_17E4);

    #[doc = include_str!("enclosing_component.doc/registration.md")]
    #[must_use]
    pub fn registration() -> ComponentRegistration {
        ComponentRegistration::new(Box::new(OuterFactory), ComponentVersion::INITIAL)
    }

    #[doc = include_str!("enclosing_component.doc/view.md")]
    #[must_use]
    pub fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(Self::ID)]))
    }
}

/// Renders one `<Guarded step="2"/>` — a child that always fails to
/// decode — so the failure originates *below* this component.
struct Outer;

/// Props for a component that takes none: it decodes from anything.
struct NoProps;

impl FromProps for NoProps {
    fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
        Ok(Self)
    }
}

impl Component for Outer {
    type Props = NoProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![GuardedComponent::element(
            ViewValue::Text("2".into()),
        )]))
    }
    fn fallback(&self) -> Option<View> {
        Some(GuardedComponent::text_view("outer fallback"))
    }
}

/// Mints the one [`Outer`]; it carries no configuration.
#[derive(Debug)]
struct OuterFactory;

impl ComponentFactory for OuterFactory {
    fn component_id(&self) -> ComponentId {
        EnclosingComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(EnclosingComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, EnclosingComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Outer)
    }
}

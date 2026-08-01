#![doc = include_str!("guarded_component.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::boundary::ErrorBoundaryPolicy;
use obscur_view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, InstanceId,
};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewValue};

use crate::host::{CounterComponent, CounterProps};

#[doc = include_str!("guarded_component.doc/GuardedComponent.md")]
#[derive(Debug)]
pub struct GuardedComponent;

impl GuardedComponent {
    #[doc = include_str!("guarded_component.doc/ID.md")]
    pub const ID: ComponentId = ComponentId::from_raw(0x0BAD_C0DE);

    #[doc = include_str!("guarded_component.doc/registration.md")]
    #[must_use]
    pub fn registration(
        policy: ErrorBoundaryPolicy,
        fallback: Option<View>,
    ) -> ComponentRegistration {
        ComponentRegistration::new(
            Box::new(GuardedFactory { policy, fallback }),
            ComponentVersion::INITIAL,
        )
    }

    #[doc = include_str!("guarded_component.doc/view.md")]
    #[must_use]
    pub fn view(step: ViewValue) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(step)]))
    }

    #[doc = include_str!("guarded_component.doc/element.md")]
    #[must_use]
    pub fn element(step: ViewValue) -> ViewNode {
        ViewNode::element(Self::ID).with_prop(obscur_view::view::ViewProperty::new(
            CounterComponent::STEP,
            step,
        ))
    }

    #[doc = include_str!("guarded_component.doc/text_view.md")]
    #[must_use]
    pub fn text_view(content: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(content)]))
    }
}

/// A component with the same props as [`CounterComponent`] — so the same bad
/// `step` fails it — whose boundary policy and fallback the test picks.
struct Guarded {
    policy: ErrorBoundaryPolicy,
    fallback: Option<View>,
}

impl Component for Guarded {
    type Props = CounterProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        GuardedComponent::text_view("healthy")
    }
    fn error_boundary(&self) -> ErrorBoundaryPolicy {
        self.policy
    }
    fn fallback(&self) -> Option<View> {
        self.fallback.clone()
    }
}

/// Mints [`Guarded`]s carrying the caller's policy and fallback.
#[derive(Debug)]
struct GuardedFactory {
    policy: ErrorBoundaryPolicy,
    fallback: Option<View>,
}

impl ComponentFactory for GuardedFactory {
    fn component_id(&self) -> ComponentId {
        GuardedComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(GuardedComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, GuardedComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Guarded {
            policy: self.policy,
            fallback: self.fallback.clone(),
        })
    }
}

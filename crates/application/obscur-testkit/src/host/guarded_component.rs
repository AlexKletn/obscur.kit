//! The component that fails on purpose, so a §1.7 boundary has something to
//! contain.

mod guarded;
mod guarded_factory;

use guarded_factory::GuardedFactory;

use obscur_identity::ComponentId;
use obscur_view::boundary::ErrorBoundaryPolicy;
use obscur_view::component::{ComponentRegistration, ComponentVersion};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewValue};

use crate::host::CounterComponent;

#[doc = include_str!("../../docs/api/host/guarded_component.doc/GuardedComponent.md")]
#[derive(Debug)]
pub struct GuardedComponent;

impl GuardedComponent {
    /// The kind a view element names to mount this component.
    pub const ID: ComponentId = ComponentId::from_raw(0x0BAD_C0DE);

    /// A registration for this component under `policy`, with `fallback` as the UI its
    /// boundary shows once the budget is spent.
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

    /// A view with one `<Guarded step=…/>` element. Pass a
    /// [`ViewValue::Int`](obscur_view::view::ViewValue::Int) for the healthy path and a
    /// [`ViewValue::Text`](obscur_view::view::ViewValue::Text) for the failing one.
    #[must_use]
    pub fn view(step: ViewValue) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(step)]))
    }

    /// The bare element, for a caller that needs to nest it.
    #[must_use]
    pub fn element(step: ViewValue) -> ViewNode {
        ViewNode::element(Self::ID).with_prop(obscur_view::view::ViewProperty::new(
            CounterComponent::STEP,
            step,
        ))
    }

    /// A view that is a single text node — every fallback in these tests.
    #[must_use]
    pub fn text_view(content: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(content)]))
    }
}

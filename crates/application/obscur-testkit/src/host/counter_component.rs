#![doc = include_str!("counter_component.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol, SignalId};
use obscur_view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId, PropsError,
    decode_required,
};
use obscur_view::state::LocalState;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("counter_component.doc/CounterComponent.md")]
#[derive(Debug)]
pub struct CounterComponent;

impl CounterComponent {
    #[doc = include_str!("counter_component.doc/ID.md")]
    pub const ID: ComponentId = ComponentId::from_raw(0x00C0_FFEE);

    #[doc = include_str!("counter_component.doc/STEP.md")]
    pub const STEP: InternedSymbol = InternedSymbol::from_raw(0x8000_0021);

    #[doc = include_str!("counter_component.doc/registration.md")]
    #[must_use]
    pub fn registration(self_updates: u32, tracks: Option<SignalId>) -> ComponentRegistration {
        ComponentRegistration::new(
            Box::new(CounterFactory {
                self_updates,
                tracks,
            }),
            ComponentVersion::INITIAL,
        )
    }

    #[doc = include_str!("counter_component.doc/view.md")]
    #[must_use]
    pub fn view(step_by: i64) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(
            ViewValue::Int(step_by),
        )]))
    }

    #[doc = include_str!("counter_component.doc/element.md")]
    #[must_use]
    pub fn element(step: ViewValue) -> ViewNode {
        ViewNode::element(Self::ID).with_prop(ViewProperty::new(Self::STEP, step))
    }
}

#[doc = include_str!("counter_component.doc/CounterProps.md")]
pub struct CounterProps {
    #[doc = include_str!("counter_component.doc/step.md")]
    pub step: u32,
}

impl FromProps for CounterProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        Ok(Self {
            step: decode_required(props, CounterComponent::STEP, "step")?,
        })
    }
}

/// Adds `step` to its own local state on every render and emits the total
/// as a text child — so the text reveals both the decoded prop and whether
/// the state survived.
struct Counter {
    total: LocalState<u32>,
    /// Ask for exactly one more render, then settle — modelling a
    /// component that advances its own state once.
    self_updates: u32,
    /// The observable source this component reads, if any. `None` models
    /// a component that depends on no app state at all.
    tracks: Option<SignalId>,
}

impl Component for Counter {
    type Props = CounterProps;
    fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        if let Some(source) = self.tracks {
            ctx.track(source);
        }
        self.total.set(*self.total.get() + props.step);
        if self.self_updates > 0 {
            self.self_updates -= 1;
            ctx.request_update();
        }
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(
            self.total.get().to_string(),
        )]))
    }
}

/// Mints [`Counter`]s carrying the caller's self-update budget and tracked
/// source.
#[derive(Debug)]
struct CounterFactory {
    self_updates: u32,
    tracks: Option<SignalId>,
}

impl ComponentFactory for CounterFactory {
    fn component_id(&self) -> ComponentId {
        CounterComponent::ID
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(CounterComponent::ID)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, CounterComponent::ID)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Counter {
            total: LocalState::new(0),
            self_updates: self.self_updates,
            tracks: self.tracks,
        })
    }
}

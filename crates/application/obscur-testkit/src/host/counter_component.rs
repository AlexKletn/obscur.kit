//! A component fixture for testing prop decoding and retained state.

mod counter;
mod counter_factory;

use counter_factory::CounterFactory;

use obscur_identity::{ComponentId, InternedSymbol, SignalId};
use obscur_view::component::{ComponentRegistration, ComponentVersion};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/host/counter_component.doc/CounterComponent.md")]
#[derive(Debug)]
pub struct CounterComponent;

impl CounterComponent {
    /// The kind a view element names to mount this component.
    pub const ID: ComponentId = ComponentId::from_raw(0x00C0_FFEE);

    /// The `step` prop symbol, minted in the non-reserved range so it resolves the way
    /// an author's own property would.
    pub const STEP: InternedSymbol = InternedSymbol::from_raw(0x8000_0021);

    /// A registration for this component, carrying the self-update budget and the
    /// tracked source described on [`CounterComponent`].
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

    /// A view with one `<Counter step=N/>` element.
    #[must_use]
    pub fn view(step_by: i64) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::element(
            ViewValue::Int(step_by),
        )]))
    }

    /// The bare element, so a caller can put two of them side by side — or hand `step`
    /// a value that will *not* decode.
    #[must_use]
    pub fn element(step: ViewValue) -> ViewNode {
        ViewNode::element(Self::ID).with_prop(ViewProperty::new(Self::STEP, step))
    }
}

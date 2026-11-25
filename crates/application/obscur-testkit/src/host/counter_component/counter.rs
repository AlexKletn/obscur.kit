use obscur_identity::SignalId;
use obscur_view::component::{Component, ComponentContext};
use obscur_view::state::LocalState;
use obscur_view::view::{View, ViewFragment, ViewNode};

use crate::host::CounterProps;

/// Adds `step` to its own local state on every render and emits the total
/// as a text child — so the text reveals both the decoded prop and whether
/// the state survived.
pub(super) struct Counter {
    pub(super) total: LocalState<u32>,
    /// Ask for exactly one more render, then settle — modelling a
    /// component that advances its own state once.
    pub(super) self_updates: u32,
    /// The observable source this component reads, if any. `None` models
    /// a component that depends on no app state at all.
    pub(super) tracks: Option<SignalId>,
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

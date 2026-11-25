use obscur_view::boundary::ErrorBoundaryPolicy;
use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::View;

use crate::host::CounterProps;

use super::GuardedComponent;

/// A component with the same props as [`CounterComponent`](crate::host::CounterComponent) — so the same bad
/// `step` fails it — whose boundary policy and fallback the test picks.
pub(super) struct Guarded {
    pub(super) policy: ErrorBoundaryPolicy,
    pub(super) fallback: Option<View>,
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

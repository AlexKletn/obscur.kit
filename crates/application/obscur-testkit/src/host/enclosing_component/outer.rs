use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::{View, ViewFragment, ViewValue};

use crate::host::GuardedComponent;

use super::no_props::NoProps;

/// Renders one `<Guarded step="2"/>` — a child that always fails to
/// decode — so the failure originates *below* this component.
pub(super) struct Outer;

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

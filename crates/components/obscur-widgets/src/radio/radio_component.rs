#![doc = include_str!("../../docs/api/radio/radio_component.doc/_module.md")]

use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::View;

use super::{Radio, RadioProps};

#[doc = include_str!("../../docs/api/radio/radio_component.doc/RadioComponent.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RadioComponent;

impl Component for RadioComponent {
    /// The authored surface: [`RadioProps`](super::RadioProps).
    type Props = RadioProps;

    #[doc = include_str!("../../docs/api/radio/radio_component.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(Radio::contents(props.selected, &props.label))
    }
}

#[cfg(test)]
mod tests {
    use super::RadioComponent;
    use crate::radio::{Radio, RadioProps};
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{Component, ComponentContext};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_view::state::{SignalGenerations, SubscriptionScope};
    use obscur_view::view::{View, ViewNode};

    fn rendered(props: &RadioProps) -> View {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        RadioComponent.render(&mut ctx, props)
    }

    #[test]
    fn renders_the_children_the_builder_installs_not_a_second_control() {
        // The whole design in one assertion: the component's output is what
        // `view()` puts UNDER the root, because the runtime reconciles it into a
        // host node that already IS the radio.
        let props = RadioProps {
            label: "One".into(),
            selected: true,
        };
        let mounted = rendered(&props);
        let built = Radio::new("One", InternedSymbol::from_raw(0x5678))
            .selected(true)
            .view();
        let ViewNode::Element { children, .. } = &built.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(mounted.root(), children);
        // …and it is not a nested `<radio>`.
        let ViewNode::Element { component, .. } = &mounted.root().nodes()[0] else {
            panic!("expected element first child");
        };
        assert_ne!(*component, Radio::COMPONENT_ID);
    }

    #[test]
    fn the_dot_carries_the_fill_only_when_selected() {
        for (selected, fills) in [(true, 1), (false, 0)] {
            let view = rendered(&RadioProps {
                label: "A".into(),
                selected,
            });
            let ViewNode::Element {
                props, children, ..
            } = &view.root().nodes()[0]
            else {
                panic!("expected the indicator element");
            };
            let class = props
                .iter()
                .find(|p| p.name() == obscur_view::view::CLASS)
                .expect("the indicator carries a class");
            assert_eq!(
                *class.value(),
                obscur_view::view::ViewValue::from("radio-dot")
            );
            assert_eq!(children.nodes().len(), fills, "selected {selected}");
        }
    }
}

#![doc = include_str!("checkbox_component.doc/_module.md")]

use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::View;

use super::{Checkbox, CheckboxProps};

#[doc = include_str!("checkbox_component.doc/CheckboxComponent.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CheckboxComponent;

impl Component for CheckboxComponent {
    #[doc = include_str!("checkbox_component.doc/Props.md")]
    type Props = CheckboxProps;

    #[doc = include_str!("checkbox_component.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(Checkbox::contents(props.checked, &props.label))
    }
}

#[cfg(test)]
mod tests {
    use super::CheckboxComponent;
    use crate::checkbox::{Checkbox, CheckboxProps};
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{Component, ComponentContext};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_view::state::{SignalGenerations, SubscriptionScope};
    use obscur_view::view::{View, ViewNode};

    fn rendered(props: &CheckboxProps) -> View {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        CheckboxComponent.render(&mut ctx, props)
    }

    #[test]
    fn renders_the_children_the_builder_installs_not_a_second_control() {
        // The whole design in one assertion: the component's output is what
        // `view()` puts UNDER the root, because the runtime reconciles it into a
        // host node that already IS the checkbox.
        let props = CheckboxProps {
            label: "Agree".into(),
            checked: Some(true),
        };
        let mounted = rendered(&props);
        let built = Checkbox::new("Agree", InternedSymbol::from_raw(0x1234))
            .checked(true)
            .view();
        let ViewNode::Element { children, .. } = &built.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(mounted.root(), children);
        // …and it is not a nested `<checkbox>`.
        let ViewNode::Element { component, .. } = &mounted.root().nodes()[0] else {
            panic!("expected element first child");
        };
        assert_ne!(*component, Checkbox::COMPONENT_ID);
    }

    #[test]
    fn the_indicator_carries_the_state_class_for_every_tri_state() {
        for (checked, expected) in [
            (Some(true), "checkbox-box checked"),
            (Some(false), "checkbox-box"),
            (None, "checkbox-box indeterminate"),
        ] {
            let view = rendered(&CheckboxProps {
                label: "A".into(),
                checked,
            });
            let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
                panic!("expected the indicator element");
            };
            let class = props
                .iter()
                .find(|p| p.name() == obscur_view::view::CLASS)
                .expect("the indicator carries a class");
            assert_eq!(
                *class.value(),
                obscur_view::view::ViewValue::from(expected),
                "{checked:?}"
            );
        }
    }
}

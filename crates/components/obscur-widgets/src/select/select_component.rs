#![doc = include_str!("select_component.doc/_module.md")]

use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::View;

use super::{Select, SelectProps};

#[doc = include_str!("select_component.doc/SelectComponent.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectComponent;

impl Component for SelectComponent {
    #[doc = include_str!("select_component.doc/Props.md")]
    type Props = SelectProps;

    #[doc = include_str!("select_component.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(Select::contents(&props.options, props.selected, props.open))
    }
}

#[cfg(test)]
mod tests {
    use super::SelectComponent;
    use crate::select::{Select, SelectProps};
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{Component, ComponentContext};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_view::state::{SignalGenerations, SubscriptionScope};
    use obscur_view::view::{CLASS, View, ViewNode, ViewValue};

    fn rendered(props: &SelectProps) -> View {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        SelectComponent.render(&mut ctx, props)
    }

    fn sample(open: bool) -> SelectProps {
        SelectProps {
            options: vec!["One".into(), "Two".into()],
            selected: Some(1),
            open,
        }
    }

    #[test]
    fn renders_the_children_the_builder_installs_not_a_second_control() {
        // The whole design in one assertion: the component's output is what
        // `view()` puts UNDER the root, because the runtime reconciles it into a
        // host node that already IS the select.
        let mut built = Select::new(InternedSymbol::from_raw(0x9ABC)).with_options(["One", "Two"]);
        assert!(built.select(1), "the second option must be selectable");
        built.open();
        let view = built.view();
        let ViewNode::Element { children, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(rendered(&sample(true)).root(), children);
        // …and it is not a nested `<select>`.
        let mounted = rendered(&sample(true));
        let ViewNode::Element { component, .. } = &mounted.root().nodes()[0] else {
            panic!("expected element first child");
        };
        assert_ne!(*component, Select::COMPONENT_ID);
    }

    #[test]
    fn the_popup_is_present_exactly_when_open() {
        assert_eq!(
            rendered(&sample(false)).root().nodes().len(),
            1,
            "closed: header only"
        );
        assert_eq!(
            rendered(&sample(true)).root().nodes().len(),
            2,
            "open: header + list"
        );
    }

    #[test]
    fn a_disabled_control_still_renders_the_popup_it_was_disabled_with() {
        // The one state the UA sheet's `select:disabled .select-option` rule can
        // match. `Select::open` refuses on a disabled control, so this state is
        // only ever reached by disabling a control whose popup is already down —
        // and `disabled` is not a prop of this component precisely so that the
        // refusal, which is a rule about a transition, cannot leak into the render
        // of a state and delete those rows.
        let view = rendered(&sample(true));
        let ViewNode::Element { children, .. } = &view.root().nodes()[1] else {
            panic!("expected the .select-list element");
        };
        assert_eq!(children.nodes().len(), 2, "one row per option");
        let ViewNode::Element { props, .. } = &children.nodes()[1] else {
            panic!("expected an option row");
        };
        assert!(
            props.iter().any(|prop| prop.name() == CLASS
                && *prop.value() == ViewValue::from("select-option selected")),
            "the picked row keeps its class while the control is disabled"
        );
    }

    #[test]
    fn a_selection_past_the_end_falls_back_to_the_first_option() {
        // Nothing rejects an out-of-range index (see `SelectProps::from_props`), so
        // state what it renders: the header shows the first option and no row is
        // marked, which is what a bare `<select>` shows.
        let view = rendered(&SelectProps {
            options: vec!["One".into(), "Two".into()],
            selected: Some(7),
            open: true,
        });
        let ViewNode::Element { children, .. } = &view.root().nodes()[0] else {
            panic!("expected the .select-header element");
        };
        let ViewNode::Text { content, .. } = &children.nodes()[0] else {
            panic!("expected the header label");
        };
        assert_eq!(&**content, "One");
    }
}

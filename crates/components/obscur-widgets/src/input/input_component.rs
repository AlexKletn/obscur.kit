#![doc = include_str!("../../docs/api/input/input_component.doc/_module.md")]

use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::View;

use super::{Input, InputProps};

#[doc = include_str!("../../docs/api/input/input_component.doc/InputComponent.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputComponent;

impl Component for InputComponent {
    /// The authored surface: [`InputProps`](super::InputProps).
    type Props = InputProps;

    #[doc = include_str!("../../docs/api/input/input_component.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::from_fragment(Input::contents(&props.value))
    }
}

#[cfg(test)]
mod tests {
    use super::InputComponent;
    use crate::input::{Input, InputProps};
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{Component, ComponentContext};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_view::state::{SignalGenerations, SubscriptionScope};
    use obscur_view::view::{View, ViewNode};

    fn rendered(props: &InputProps) -> View {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        InputComponent.render(&mut ctx, props)
    }

    #[test]
    fn renders_the_children_the_builder_installs_not_a_second_control() {
        // The whole design in one assertion: the component's output is what
        // `view()` puts UNDER the root, because the runtime reconciles it into a
        // host node that already IS the field.
        let props = InputProps {
            value: "hello".into(),
        };
        let mounted = rendered(&props);
        let built = Input::with_value(InternedSymbol::from_raw(0x5678), "hello").view();
        let ViewNode::Element { children, .. } = &built.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(mounted.root(), children);
    }

    #[test]
    fn the_value_is_a_single_text_child_in_every_state() {
        // The editing session auto-seeds from the focused editable's descendant
        // `Text` content, and the caret indexes that one run — so an empty field
        // must still render exactly one (empty) text node, not zero. This is the
        // shape the runtime's whole editing path is written against.
        for value in ["", "hello", "line one\nline two"] {
            let view = rendered(&InputProps {
                value: value.into(),
            });
            assert_eq!(view.root().nodes().len(), 1, "{value:?}");
            let ViewNode::Text { content, .. } = &view.root().nodes()[0] else {
                panic!("expected a single text child for {value:?}");
            };
            assert_eq!(&**content, value);
        }
    }

    #[test]
    fn a_multiline_field_renders_the_same_body_as_a_single_line_one() {
        // `<input>` and `<textarea>` are one component, and the difference between
        // them is not in this output: it rides on the host node's local name,
        // which a component render never touches. `Input::multiline` changes the
        // ROOT the builder emits, not the body — so the body is the same fragment
        // for both, and the props struct needs no `multiline` field.
        let command = InternedSymbol::from_raw(0x5678);
        let area = Input::with_value(command, "hi").multiline(true).view();
        let field = Input::with_value(command, "hi").view();
        let body = |view: &View| match &view.root().nodes()[0] {
            ViewNode::Element { children, .. } => children.clone(),
            ViewNode::Text { .. } => panic!("expected element root"),
        };
        assert_eq!(body(&area), body(&field));
        assert_eq!(
            *rendered(&InputProps { value: "hi".into() }).root(),
            body(&area)
        );
    }
}

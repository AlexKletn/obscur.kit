#![doc = include_str!("../../docs/api/button/button_component.doc/_module.md")]

use obscur_view::component::{Component, ComponentContext};
use obscur_view::view::{View, ViewNode};

use super::{Button, ButtonProps};

#[doc = include_str!("../../docs/api/button/button_component.doc/ButtonComponent.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ButtonComponent;

impl Component for ButtonComponent {
    /// The authored surface: [`ButtonProps`](super::ButtonProps).
    type Props = ButtonProps;

    #[doc = include_str!("../../docs/api/button/button_component.doc/render.md")]
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
        View::single(ViewNode::slot("").with_children(Button::contents(&props.label)))
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonComponent;
    use crate::button::{Button, ButtonProps};
    use obscur_identity::InternedSymbol;
    use obscur_view::component::{Component, ComponentContext};
    use obscur_view::lifecycle::LifecycleState;
    use obscur_view::state::{SignalGenerations, SubscriptionScope};
    use obscur_view::view::{View, ViewFragment, ViewNode, is_outlet, project_slots};

    fn rendered(label: &str) -> View {
        let mut scope = SubscriptionScope::new();
        let mut needs_update = false;
        let signals = SignalGenerations::new();
        let mut ctx = ComponentContext::new(
            &mut scope,
            LifecycleState::Mounted,
            &mut needs_update,
            &signals,
        );
        ButtonComponent.render(
            &mut ctx,
            &ButtonProps {
                label: label.to_owned(),
            },
        )
    }

    /// The text runs a mounted button leaves behind, given `content` as the
    /// host's authored children — the runtime's own two steps, in order.
    fn mounted(label: &str, content: &ViewFragment) -> Vec<String> {
        project_slots(rendered(label), content)
            .root()
            .nodes()
            .iter()
            .map(|node| match node {
                ViewNode::Text { content, .. } => content.to_string(),
                ViewNode::Element { .. } => panic!("a button's content is text here"),
            })
            .collect()
    }

    #[test]
    fn renders_one_position_and_it_is_an_outlet() {
        // The whole design in one assertion. Every other shipped control renders
        // content of its own; this one renders *where the author's content goes*,
        // and nothing else — which is why mounting it cannot replace markup.
        let view = rendered("Save");
        assert_eq!(view.root().len(), 1);
        assert!(is_outlet(&view.root().nodes()[0]));
    }

    #[test]
    fn the_authored_content_wins_and_the_label_is_the_fallback() {
        // The campaign's acceptance test, at the level where the answer lives.
        let authored = ViewFragment::from_nodes(vec![ViewNode::text("Save")]);
        assert_eq!(mounted("", &authored), ["Save"], "`<button>Save</button>`");
        assert_eq!(
            mounted("Save", &ViewFragment::new()),
            ["Save"],
            "`<button label=\"Save\"/>`"
        );
        assert_eq!(
            mounted("Ignored", &authored),
            ["Save"],
            "both spellings at once: the author's content is never displaced"
        );
        assert!(
            mounted("", &ViewFragment::new()).is_empty(),
            "and a button with neither renders nothing, not an empty text run"
        );
    }

    #[test]
    fn mounting_reproduces_the_children_the_builder_installs() {
        // The idempotence proof for the Rust path: the node `Button::view` builds
        // carries its label as a child, so the component projects that child back
        // and the tree is the one that was already there.
        let built = Button::new("Save", InternedSymbol::from_raw(0xABCD)).view();
        let ViewNode::Element { children, .. } = &built.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(
            project_slots(rendered(""), children).root(),
            children,
            "mounting a built button changes nothing about it"
        );
    }
}

//! The panel two pointers can work at once.

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol, NodeId};
use obscur_runtime::UiRuntime;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::Stylesheet;
use obscur_view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use crate::fixture::{ClassRule, TestFixture};
use crate::selector::NodePath;

#[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/InteractionPanel.md")]
#[derive(Debug)]
pub struct InteractionPanel {
    /// The runtime, with one frame already rendered.
    pub runtime: UiRuntime,
    /// The container node, y ∈ [0, 100).
    pub panel: NodeId,
    /// The upper child, y ∈ [0, 50), bound to [`CMD_A`](Self::CMD_A).
    pub a: NodeId,
    /// The lower child, y ∈ [50, 100), bound to [`CMD_B`](Self::CMD_B).
    pub b: NodeId,
    /// Set when `a`'s handler runs.
    pub ran_a: Rc<Cell<bool>>,
    /// Set when `b`'s handler runs.
    pub ran_b: Rc<Cell<bool>>,
}

impl InteractionPanel {
    /// The click command bound to the upper child.
    pub const CMD_A: InternedSymbol = InternedSymbol::from_raw(0x0A);

    /// The click command bound to the lower child.
    pub const CMD_B: InternedSymbol = InternedSymbol::from_raw(0x0B);

    #[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        let mut runtime = TestFixture::runtime();
        let (ran_a, ran_b) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
        {
            let (a, b) = (ran_a.clone(), ran_b.clone());
            runtime.on_command(Self::CMD_A, move || a.set(true));
            runtime.on_command(Self::CMD_B, move || b.set(true));
        }
        let sheet = Self::sheet();
        let _ = runtime.render_now(
            &Self::command_view(Self::CMD_A, Self::CMD_B),
            std::slice::from_ref(&sheet),
        );
        let (panel, a, b) = Self::nodes(&runtime);
        Self {
            runtime,
            panel,
            a,
            b,
            ran_a,
            ran_b,
        }
    }

    #[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/view.md")]
    #[must_use]
    pub fn view() -> View {
        Self::panel(Self::child)
    }

    /// The panel with each child bound to the given click command.
    #[must_use]
    pub fn command_view(cmd_a: InternedSymbol, cmd_b: InternedSymbol) -> View {
        Self::panel(|component, class| {
            let command = if component == 101 { cmd_a } else { cmd_b };
            Self::child(component, class).with_event(ViewEventBinding::new(EVENT_CLICK, command))
        })
    }

    #[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/ancestor_bound_view.md")]
    #[must_use]
    pub fn ancestor_bound_view(panel_command: InternedSymbol, b_command: InternedSymbol) -> View {
        let node = Self::panel_node(move |component, class| {
            let child = Self::child(component, class);
            if component == 102 {
                child.with_event(ViewEventBinding::new(EVENT_CLICK, b_command))
            } else {
                child
            }
        })
        .with_event(ViewEventBinding::new(EVENT_CLICK, panel_command));
        View::from_fragment(ViewFragment::from_nodes(vec![node]))
    }

    #[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/sheet.md")]
    #[must_use]
    pub fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                ClassRule::new("panel").sized(100.0, 100.0).build(),
                ClassRule::new("a").sized(100.0, 50.0).build(),
                ClassRule::new("b").sized(100.0, 50.0).build(),
                ClassRule::hover("b").background(1.0, 0.0, 0.0).build(),
                ClassRule::focus("a").background(1.0, 1.0, 0.0).build(),
            ],
        )
    }

    /// The `(panel, a, b)` ids, after a frame has populated the tree.
    #[must_use]
    pub fn nodes(runtime: &UiRuntime) -> (NodeId, NodeId, NodeId) {
        let panel = NodePath::new(&[0]).resolve(runtime);
        let kids = NodePath::new(&[0]).children(runtime);
        (panel, kids[0], kids[1])
    }

    /// Builds the container and its two children through `child`.
    fn panel(child: impl Fn(u64, &str) -> ViewNode) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![Self::panel_node(child)]))
    }

    #[doc = include_str!("../../docs/api/fixture/interaction_panel.doc/panel_node.md")]
    fn panel_node(child: impl Fn(u64, &str) -> ViewNode) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(100))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                child(101, "a"),
                child(102, "b"),
            ]))
    }

    /// One classed child element.
    fn child(component: u64, class: &str) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(component))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    }
}

impl Default for InteractionPanel {
    fn default() -> Self {
        Self::new()
    }
}

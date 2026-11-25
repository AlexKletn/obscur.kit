//! The panel that tells a hold from a tap.

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol, NodeId};
use obscur_runtime::UiRuntime;
use obscur_style::matching::Stylesheet;
use obscur_view::view::{
    CLASS, EVENT_CLICK, EVENT_LONG_PRESS, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};

use crate::fixture::{InteractionPanel, TestFixture};

#[doc = include_str!("../../docs/api/fixture/long_press_panel.doc/LongPressPanel.md")]
#[derive(Debug)]
pub struct LongPressPanel {
    /// The runtime, with one frame already rendered.
    pub runtime: UiRuntime,
    /// The view the fixture rendered, so a test can run further frames over it.
    pub view: View,
    /// The sheet the fixture rendered, alongside [`view`](Self::view).
    pub sheet: Stylesheet,
    /// `a` — bound to a click only, so a contact on it arms no hold.
    pub tappable: NodeId,
    /// `b` — bound to both a click and a long press.
    pub holdable: NodeId,
    /// Set when the click command's handler runs.
    pub clicked: Rc<Cell<bool>>,
    /// Set when the long-press command's handler runs.
    pub held: Rc<Cell<bool>>,
}

impl LongPressPanel {
    /// The click command both children bind.
    pub const CMD_CLICK: InternedSymbol = InternedSymbol::from_raw(0x1A);

    /// The long-press command only `b` binds.
    pub const CMD_HOLD: InternedSymbol = InternedSymbol::from_raw(0x1B);

    /// A y inside `b`, the child that binds the hold.
    pub const HOLD_TARGET_Y: f32 = 75.0;

    /// Renders the panel and registers a handler for each of the two commands.
    #[must_use]
    pub fn new() -> Self {
        let mut runtime = TestFixture::runtime();
        let (clicked, held) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
        {
            let (c, h) = (clicked.clone(), held.clone());
            runtime.on_command(Self::CMD_CLICK, move || c.set(true));
            runtime.on_command(Self::CMD_HOLD, move || h.set(true));
        }
        let view = Self::build_view();
        let sheet = InteractionPanel::sheet();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let (_panel, tappable, holdable) = InteractionPanel::nodes(&runtime);
        Self {
            runtime,
            view,
            sheet,
            tappable,
            holdable,
            clicked,
            held,
        }
    }

    /// The panel with `a` click-bound and `b` click- and hold-bound.
    fn build_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(100))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(vec![
                    ViewNode::element(ComponentId::from_raw(101))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                        .with_event(ViewEventBinding::new(EVENT_CLICK, Self::CMD_CLICK)),
                    ViewNode::element(ComponentId::from_raw(102))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                        .with_event(ViewEventBinding::new(EVENT_CLICK, Self::CMD_CLICK))
                        .with_event(ViewEventBinding::new(EVENT_LONG_PRESS, Self::CMD_HOLD)),
                ])),
        ]))
    }
}

impl Default for LongPressPanel {
    fn default() -> Self {
        Self::new()
    }
}

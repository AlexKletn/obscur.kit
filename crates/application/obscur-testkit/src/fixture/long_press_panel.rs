#![doc = include_str!("long_press_panel.doc/_module.md")]

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

#[doc = include_str!("long_press_panel.doc/LongPressPanel.md")]
#[derive(Debug)]
pub struct LongPressPanel {
    #[doc = include_str!("long_press_panel.doc/runtime.md")]
    pub runtime: UiRuntime,
    #[doc = include_str!("long_press_panel.doc/view.md")]
    pub view: View,
    #[doc = include_str!("long_press_panel.doc/sheet.md")]
    pub sheet: Stylesheet,
    #[doc = include_str!("long_press_panel.doc/tappable.md")]
    pub tappable: NodeId,
    #[doc = include_str!("long_press_panel.doc/holdable.md")]
    pub holdable: NodeId,
    #[doc = include_str!("long_press_panel.doc/clicked.md")]
    pub clicked: Rc<Cell<bool>>,
    #[doc = include_str!("long_press_panel.doc/held.md")]
    pub held: Rc<Cell<bool>>,
}

impl LongPressPanel {
    #[doc = include_str!("long_press_panel.doc/CMD_CLICK.md")]
    pub const CMD_CLICK: InternedSymbol = InternedSymbol::from_raw(0x1A);

    #[doc = include_str!("long_press_panel.doc/CMD_HOLD.md")]
    pub const CMD_HOLD: InternedSymbol = InternedSymbol::from_raw(0x1B);

    #[doc = include_str!("long_press_panel.doc/HOLD_TARGET_Y.md")]
    pub const HOLD_TARGET_Y: f32 = 75.0;

    #[doc = include_str!("long_press_panel.doc/new.md")]
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

    #[doc = include_str!("long_press_panel.doc/build_view.md")]
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

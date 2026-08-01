#![doc = include_str!("editable_panel.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::UiRuntime;
use obscur_style::matching::Stylesheet;
use obscur_view::view::{
    CLASS, EVENT_INPUT, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use crate::fixture::{InteractionPanel, TestFixture, mouse};

#[doc = include_str!("editable_panel.doc/EditablePanel.md")]
#[derive(Debug)]
pub struct EditablePanel;

impl EditablePanel {
    #[doc = include_str!("editable_panel.doc/CMD_INPUT.md")]
    pub const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x1247);

    #[doc = include_str!("editable_panel.doc/view.md")]
    #[must_use]
    pub fn view(text: &str) -> View {
        Self::panel(vec![Self::field().with_children(ViewFragment::from_nodes(
            vec![ViewNode::text(text.to_string())],
        ))])
    }

    #[doc = include_str!("editable_panel.doc/placeholder_view.md")]
    #[must_use]
    pub fn placeholder_view() -> View {
        Self::panel(vec![Self::field().with_children(ViewFragment::from_nodes(
            vec![
                ViewNode::element(ComponentId::from_raw(502))
                    .with_prop(ViewProperty::new(
                        CLASS,
                        ViewValue::Text("placeholder".into()),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        "Type here…".to_string(),
                    )])),
            ],
        ))])
    }

    #[doc = include_str!("editable_panel.doc/sheet.md")]
    #[must_use]
    pub fn sheet() -> Stylesheet {
        InteractionPanel::sheet()
    }

    #[doc = include_str!("editable_panel.doc/focused.md")]
    #[must_use]
    pub fn focused(seed: &str) -> (UiRuntime, Rc<RefCell<String>>) {
        let mut runtime = TestFixture::runtime();
        let mirror = Rc::new(RefCell::new(String::new()));
        {
            let s = mirror.clone();
            runtime.on_text_input(Self::CMD_INPUT, move |resulting| {
                resulting.clone_into(&mut s.borrow_mut());
            });
        }
        let sheet = Self::sheet();
        let sheets = std::slice::from_ref(&sheet);
        let _ = runtime.render_now(&Self::view(seed), sheets);
        // Press inside `b` to focus the editable; the runtime seeds the session.
        runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert!(
            runtime.focused().is_some(),
            "the press focused the editable"
        );
        (runtime, mirror)
    }

    #[doc = include_str!("editable_panel.doc/panel.md")]
    fn panel(children: Vec<ViewNode>) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(500))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(children)),
        ]))
    }

    #[doc = include_str!("editable_panel.doc/field.md")]
    fn field() -> ViewNode {
        ViewNode::element(ComponentId::from_raw(501))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
            .with_event(ViewEventBinding::new(EVENT_INPUT, Self::CMD_INPUT))
    }
}

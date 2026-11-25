//! The editable a text test focuses: one field that binds `on-input`, inside the
//! [`InteractionPanel`](super::InteractionPanel) geometry every press coordinate in
//! these tests depends on.

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

#[doc = include_str!("../../docs/api/fixture/editable_panel.doc/EditablePanel.md")]
#[derive(Debug)]
pub struct EditablePanel;

impl EditablePanel {
    /// The app-owned command the field binds `on-input` to, and that a test registers a
    /// text-input handler for.
    pub const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x1247);

    /// The field seeded with `text` as its editable value.
    #[must_use]
    pub fn view(text: &str) -> View {
        Self::panel(vec![Self::field().with_children(ViewFragment::from_nodes(
            vec![ViewNode::text(text.to_string())],
        ))])
    }

    #[doc = include_str!("../../docs/api/fixture/editable_panel.doc/placeholder_view.md")]
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

    /// The sheet that sizes the panel and its field — [`InteractionPanel::sheet`].
    #[must_use]
    pub fn sheet() -> Stylesheet {
        InteractionPanel::sheet()
    }

    #[doc = include_str!("../../docs/api/fixture/editable_panel.doc/focused.md")]
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

    /// The 100×100 `panel` wrapper both views share.
    fn panel(children: Vec<ViewNode>) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(500))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(children)),
        ]))
    }

    /// The `b` child, bound to `on-input`, before its children are attached.
    fn field() -> ViewNode {
        ViewNode::element(ComponentId::from_raw(501))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
            .with_event(ViewEventBinding::new(EVENT_INPUT, Self::CMD_INPUT))
    }
}

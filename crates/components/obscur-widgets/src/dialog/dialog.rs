#![doc = include_str!("dialog.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Portal;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("dialog.doc/Dialog.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Dialog {
    title: Box<str>,
    content: Vec<ViewNode>,
    open: bool,
    modal: bool,
}

impl Dialog {
    #[doc = include_str!("dialog.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0006);

    #[doc = include_str!("dialog.doc/PORTAL_TARGET.md")]
    const PORTAL_TARGET: &'static str = "dialog-root";

    #[doc = include_str!("dialog.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0600);
    #[doc = include_str!("dialog.doc/PROP_MODAL.md")]
    const PROP_MODAL: InternedSymbol = InternedSymbol::from_raw(0x00D1_0601);
    #[doc = include_str!("dialog.doc/PROP_TITLE.md")]
    const PROP_TITLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0602);

    #[doc = include_str!("dialog.doc/new.md")]
    #[must_use]
    pub fn new(title: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            open: false,
            modal: true,
        }
    }

    #[doc = include_str!("dialog.doc/child.md")]
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.content.push(node);
        self
    }

    #[doc = include_str!("dialog.doc/modal.md")]
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    #[doc = include_str!("dialog.doc/open.md")]
    pub fn open(&mut self) {
        self.open = true;
    }

    #[doc = include_str!("dialog.doc/close.md")]
    pub fn close(&mut self) {
        self.open = false;
    }

    #[doc = include_str!("dialog.doc/toggle_open.md")]
    pub fn toggle_open(&mut self) {
        self.open = !self.open;
    }

    #[doc = include_str!("dialog.doc/is_open.md")]
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    #[doc = include_str!("dialog.doc/is_modal.md")]
    #[must_use]
    pub const fn is_modal(&self) -> bool {
        self.modal
    }

    #[doc = include_str!("dialog.doc/title.md")]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[doc = include_str!("dialog.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Dialog
    }

    #[doc = include_str!("dialog.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        if !self.open {
            return View::empty();
        }
        let dialog_element = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_ROLE,
                ViewValue::from("dialog"),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_MODAL,
                ViewValue::Bool(self.modal),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_TITLE,
                ViewValue::Text(self.title.clone()),
            ))
            .with_children(ViewFragment::from_nodes(self.content.clone()));
        let portal_node = Portal::new(Self::PORTAL_TARGET)
            .child(dialog_element)
            .into_view_node();
        View::single(portal_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_view_is_empty() {
        let d = Dialog::new("Settings");
        assert!(!d.is_open());
        assert_eq!(d.view().root().len(), 0);
    }

    #[test]
    fn defaults_are_modal_and_closed() {
        let d = Dialog::new("Settings");
        assert!(d.is_modal());
        assert!(!d.is_open());
        assert_eq!(d.title(), "Settings");
    }

    #[test]
    fn toggle_open_cycle() {
        let mut d = Dialog::new("X");
        d.toggle_open();
        assert!(d.is_open());
        d.toggle_open();
        assert!(!d.is_open());
    }

    #[test]
    fn semantic_role_is_dialog() {
        assert_eq!(Dialog::new("X").semantic_role(), SemanticRole::Dialog);
    }

    #[test]
    fn open_view_is_portal_wrapping_dialog_element() {
        let mut d = Dialog::new("Confirm")
            .modal(false)
            .child(ViewNode::text("Are you sure?"));
        d.open();
        let view = d.view();
        assert_eq!(view.root().len(), 1);
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                props,
                children,
                ..
            } => {
                assert_eq!(*component, Portal::COMPONENT_ID);
                assert!(props.iter().any(|p| p.name() == Portal::PROP_TARGET
                    && *p.value() == ViewValue::Text("dialog-root".into())));
                assert_eq!(children.len(), 1);
                match &children.nodes()[0] {
                    ViewNode::Element {
                        component,
                        props,
                        children,
                        ..
                    } => {
                        assert_eq!(*component, Dialog::COMPONENT_ID);
                        let title = props
                            .iter()
                            .find(|p| p.name() == Dialog::PROP_TITLE)
                            .expect("title prop present");
                        assert_eq!(*title.value(), ViewValue::Text("Confirm".into()));
                        let modal = props
                            .iter()
                            .find(|p| p.name() == Dialog::PROP_MODAL)
                            .expect("modal prop present");
                        assert_eq!(*modal.value(), ViewValue::Bool(false));
                        assert_eq!(children.len(), 1);
                    }
                    ViewNode::Text { .. } => panic!("expected dialog element"),
                }
            }
            ViewNode::Text { .. } => panic!("expected portal element root"),
        }
    }
}

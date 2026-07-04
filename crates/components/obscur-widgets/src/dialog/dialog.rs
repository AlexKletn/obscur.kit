//! `Dialog`: a headless modal/modeless dialog surface (spec §2.6, §6.9).

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Portal;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A headless dialog surface hosting arbitrary content (spec §2.6, §6.9).
///
/// `Dialog` owns its `title`, an ordered list of arbitrary content
/// [`ViewNode`]s, whether it is currently [`open`](Self::is_open), and whether
/// it is [`modal`](Self::is_modal). It bakes in **no** visual theme.
///
/// Rendering is gated on the open flag: while closed, [`view`](Self::view)
/// returns [`View::empty`] — a dialog contributes nothing to the tree until it
/// is opened. While open, its element is projected through a
/// [`Portal`](obscur_primitives::Portal) to the `"dialog-root"` target so the
/// surface escapes its ancestor's clip and z-order; the element carries the
/// role, modal flag, and title as properties and hosts the content nodes as its
/// children, rendered by the style layer keyed on
/// [`COMPONENT_ID`](Self::COMPONENT_ID).
///
/// The state machine is a plain open/close toggle ([`open`](Self::open),
/// [`close`](Self::close), [`toggle_open`](Self::toggle_open)).
#[derive(Debug, Clone, PartialEq)]
pub struct Dialog {
    title: Box<str>,
    content: Vec<ViewNode>,
    open: bool,
    modal: bool,
}

impl Dialog {
    /// Stable component id for the dialog widget (see [`Button::COMPONENT_ID`]
    /// for the shared `0x00D0_00NN` scheme).
    ///
    /// [`Button::COMPONENT_ID`]: crate::button::Button::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0006);

    /// The portal target the dialog surface is projected to when open.
    const PORTAL_TARGET: &'static str = "dialog-root";

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 06` (dialog): role=`00`, modal=`01`,
    /// title=`02`.
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0600);
    /// Interned name of the `modal` state property.
    const PROP_MODAL: InternedSymbol = InternedSymbol::from_raw(0x00D1_0601);
    /// Interned name of the `title` property.
    const PROP_TITLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0602);

    /// Creates a closed, modal dialog with the given `title` and no content.
    #[must_use]
    pub fn new(title: impl Into<Box<str>>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            open: false,
            modal: true,
        }
    }

    /// Appends a content child node, returning the updated dialog (fluent
    /// builder).
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.content.push(node);
        self
    }

    /// Sets the modal flag, returning the updated dialog (fluent builder).
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Opens the dialog.
    pub fn open(&mut self) {
        self.open = true;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Toggles the dialog open/closed.
    pub fn toggle_open(&mut self) {
        self.open = !self.open;
    }

    /// Returns whether the dialog is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Returns whether the dialog is modal.
    #[must_use]
    pub const fn is_modal(&self) -> bool {
        self.modal
    }

    /// Returns the dialog title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the accessible role of this control ([`SemanticRole::Dialog`]).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Dialog
    }

    /// Derives the semantic view.
    ///
    /// While [closed](Self::is_open) the dialog renders nothing —
    /// [`View::empty`] — so it contributes no top-level nodes. While open it
    /// returns a single [`Portal`](obscur_primitives::Portal) node targeting
    /// `"dialog-root"`, wrapping a [`COMPONENT_ID`] element that carries the
    /// role, modal flag, and title as properties and hosts the content nodes as
    /// its children.
    ///
    /// [`COMPONENT_ID`]: Self::COMPONENT_ID
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
        // Outer node is the portal.
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
                // Inner node is the dialog element.
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

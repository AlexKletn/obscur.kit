#![doc = include_str!("dialog.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::focus::FOCUS_TRAP;
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

    #[doc = include_str!("dialog.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0600);
    #[doc = include_str!("dialog.doc/PROP_MODAL.md")]
    const PROP_MODAL: InternedSymbol = FOCUS_TRAP;
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
        // The dialog element is emitted at its author's position in the tree, NOT
        // wrapped in a `Portal`. It used to be wrapped in `Portal::new("dialog-root")`,
        // and that wrapper was a lie with a cost: nothing in the workspace resolves a
        // portal target name to a node, so `obscur_tree::portal::Portal::apply` — the
        // one thing that would move the surface's visual parent — has no production
        // caller. The wrapper therefore added one inert element to every open dialog
        // and told the reader the surface escaped its ancestor's clip and z-order,
        // which it never did. Emitting the element directly makes the missing
        // behaviour visible where it is missing (a dialog nested in an
        // `overflow: hidden` ancestor is clipped by it) instead of hiding it behind
        // vocabulary that looks wired. See `docs/api/12-reality.md`, row *Portals*.
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
        View::single(dialog_element)
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
    fn open_view_is_the_dialog_element_itself() {
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
            ViewNode::Text { .. } => panic!("expected the dialog element at the root"),
        }
    }

    /// The regression guard for the defect this widget used to carry: a modal
    /// dialog announced its modality under a symbol private to this file, which
    /// the runtime could not read and therefore did not act on — Tab walked
    /// straight out of an open modal into the page behind it. The property has
    /// to be the *reserved* one the runtime reconciles focus traps from, so this
    /// asserts the symbol rather than the presence of some property.
    #[test]
    fn a_modal_dialog_declares_the_reserved_focus_trap() {
        let mut d = Dialog::new("Confirm").child(ViewNode::text("Are you sure?"));
        d.open();
        let view = d.view();
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected the dialog element at the root");
        };
        let trap = props
            .iter()
            .find(|p| p.name() == FOCUS_TRAP)
            .expect("an open modal dialog declares the reserved focus trap");
        assert_eq!(*trap.value(), ViewValue::Bool(true));
    }

    /// The regression guard for the defect this widget used to carry: an open
    /// dialog must not be wrapped in a `Portal` while nothing in the workspace
    /// resolves a portal target. Re-adding the wrapper without ALSO giving
    /// `obscur_tree::portal::Portal::apply` a production caller reintroduces an
    /// element that costs a tree node and delivers nothing — see the *Portals* row
    /// of `docs/api/12-reality.md`. Written against the primitive's own
    /// `COMPONENT_ID`/`PROP_TARGET` rather than against a name, so it still bites
    /// if the wrapper comes back under a different target string.
    #[test]
    fn an_open_dialog_emits_no_unresolved_portal() {
        fn portals(node: &ViewNode) -> usize {
            match node {
                ViewNode::Text { .. } => 0,
                ViewNode::Element {
                    component,
                    props,
                    children,
                    ..
                } => {
                    let here = usize::from(
                        *component == obscur_primitives::Portal::COMPONENT_ID
                            || props
                                .iter()
                                .any(|p| p.name() == obscur_primitives::Portal::PROP_TARGET),
                    );
                    here + children.nodes().iter().map(portals).sum::<usize>()
                }
            }
        }

        let mut d = Dialog::new("Confirm").child(ViewNode::text("Are you sure?"));
        d.open();
        let view = d.view();

        let found: usize = view.root().nodes().iter().map(portals).sum();
        assert_eq!(
            found, 0,
            "a shipped widget must not portal into a target nobody resolves: \
             found {found} portal node(s) in an open dialog's view"
        );
    }
}

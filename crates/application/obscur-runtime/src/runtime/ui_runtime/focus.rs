#![doc = include_str!("focus.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::focus::{FOCUS_TRAP, FocusScope, FocusScopeId};
use obscur_tree::tree::Subtree;
use obscur_view::view::{EVENT_BLUR, EVENT_FOCUS, ViewValue};

use crate::runtime::{AncestorChain, CommandDispatch, EditableField, FocusOrder};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("focus.doc/move_focus_to.md")]
    pub(super) fn move_focus_to(&mut self, next: Option<NodeId>, cause: UiEvent) -> bool {
        if self.focused == next {
            return false;
        }
        let blurred = self.focused;
        self.focused = next;
        // `:focus-visible` is decided here and only here, from the envelope that
        // caused the move: a keyboard move shows the ring, a press does not.
        // Reading it off the cause rather than taking a parameter is what stops
        // the two ever disagreeing.
        self.focus_visible = next.is_some() && cause.origin() == EventOrigin::Keyboard;
        // Focus moved: begin/clear the runtime-owned editing session so the
        // focused editable's text/caret/selection is live for keys (and step-2
        // rendering). This is also what commits the outgoing field's `change`,
        // and it runs before `blur` is dispatched because HTML fires the two in
        // that order — a handler reading the field on blur must see the
        // committed value.
        self.sync_editing_to_focus();
        // §6.15 "Focused inputs may be scrolled into view". A field half off the
        // edge of its scroller — or behind the on-screen keyboard, which is the
        // case that matters on a device with no hardware one — is brought into
        // the visible region. A field already fully visible does not move.
        if let Some(field) =
            next.and_then(|node| EditableField::editable_ancestor(&self.tree, node))
        {
            self.scroll_into_view(field);
        }
        // `:focus` and `:focus-visible` moved with it, so the projection the
        // style phase reads is stale until this runs. It sits in the one door
        // rather than in each caller because that is what makes every mover
        // correct by construction — the pointer path recomputes again at the end
        // of its own dispatch, for the hover and press states this cannot know
        // about, and one extra projection per press is a better trade than a
        // future caller that forgets.
        self.recompute_pseudo_states();
        self.dispatch_focus_change(blurred, next, cause);
        true
    }

    #[doc = include_str!("focus.doc/focus_node.md")]
    pub fn focus_node(&mut self, node: Option<NodeId>) -> bool {
        if node.is_some_and(|id| self.tree.get(id).is_none()) {
            return false;
        }
        // An open trap refuses this the same way it refuses a press: a modal that
        // Tab cannot leave but `focus_node` can is a trap only for the keyboard,
        // and the escape it leaves open is the one an application reaches for
        // most — revealing a panel and focusing its field while a dialog is up.
        if !self.focus_trap_admits(node) {
            return false;
        }
        self.move_focus_to(node, UiEvent::detached(EventOrigin::Synthetic))
    }

    #[doc = include_str!("focus.doc/is_focus_visible.md")]
    #[must_use]
    pub const fn is_focus_visible(&self) -> bool {
        self.focus_visible
    }

    #[doc = include_str!("focus.doc/move_focus_by_tab.md")]
    pub(super) fn move_focus_by_tab(&mut self, backwards: bool, cause: UiEvent) -> bool {
        // The innermost open scope that traps, if any: a modal is exactly the
        // thing that must not let Tab wander back to the page behind it.
        let order = FocusOrder::of(&self.tree, self.focus_trap());
        // An empty ring must not blur what is focused. `step` answering `None`
        // means "there is nowhere to go", and a Tab that unfocused the field the
        // user was typing in because the screen holds one control would be worse
        // than a Tab that does nothing.
        let Some(next) = order.step(self.focused, backwards) else {
            return false;
        };
        self.move_focus_to(Some(next), cause)
    }

    #[doc = include_str!("focus.doc/focus_trap.md")]
    pub(super) fn focus_trap(&self) -> Option<&FocusScope> {
        // A non-trapping scope groups commands without owning the ring, so
        // `traps()` is the whole test.
        self.routing.scopes().iter().rev().find(|s| s.traps())
    }

    #[doc = include_str!("focus.doc/focus_trap_admits.md")]
    pub(super) fn focus_trap_admits(&self, node: Option<NodeId>) -> bool {
        self.focus_trap()
            .is_none_or(|scope| node.is_some_and(|node| scope.contains(node)))
    }

    #[doc = include_str!("focus.doc/trap_declaring_nodes.md")]
    fn trap_declaring_nodes(&self) -> Vec<NodeId> {
        // Flat, not a pre-order walk. This runs on every reconciled frame beside
        // the pointer-binding rescan, and the tree it runs over is the one a
        // virtualized list fills with rows: a walk that allocated a visited set
        // and a child vector per node would put a per-node cost on every
        // structural frame to answer a question whose answer is almost always
        // "none". Nothing here allocates unless a trap is actually declared.
        let mut nodes: Vec<NodeId> = self
            .tree
            .iter()
            .filter(|(_, retained)| {
                retained.props().iter().any(|prop| {
                    prop.name() == FOCUS_TRAP && *prop.value() != ViewValue::Bool(false)
                })
            })
            .map(|(node, _)| node)
            .collect();
        // Outermost first, so that a trap opened inside a trap is the last scope
        // on the stack and therefore the one `focus_trap` answers with. Depth is
        // the whole ordering: siblings cannot nest, so their relative order is
        // free, and the flat scan leaves them in allocation order — which is the
        // order they were built in.
        if nodes.len() > 1 {
            nodes.sort_by_key(|node| {
                AncestorChain::of(&self.tree, Some(*node))
                    .into_iter()
                    .count()
            });
        }
        nodes
    }

    #[doc = include_str!("focus.doc/declared_focus_traps.md")]
    fn declared_focus_traps(&self, declaring: Vec<NodeId>) -> Vec<FocusScope> {
        declaring
            .into_iter()
            .map(|node| {
                let mut scope = FocusScope::new(FocusScopeId::from_raw(node.raw())).trapping(true);
                for member in Subtree::new(node).descendants(&self.tree) {
                    scope.add_member(member);
                }
                scope
            })
            .collect()
    }

    #[doc = include_str!("focus.doc/reconcile_focus_traps.md")]
    pub(super) fn reconcile_focus_traps(&mut self) {
        let known = self.routing.declared_traps().to_vec();
        let declaring = self.trap_declaring_nodes();
        // The frame nothing has to do, which is nearly every frame: no trap in
        // the tree and none open. Answered before a single scope is built.
        if declaring.is_empty() && known.is_empty() {
            return;
        }
        let declared = self.declared_focus_traps(declaring);
        // Innermost first: a nested pair of traps unwinds the way it was built,
        // so each restore hands focus back to the level immediately outside it
        // rather than to the outermost saved node.
        for id in known.iter().rev().copied() {
            if declared.iter().all(|scope| scope.id() != id) {
                self.leave_focus_trap(id);
            }
        }
        for scope in &declared {
            // Re-opened whether it is new or not: `open_scope` replaces the entry
            // with this id, and the membership it replaces may be a frame stale —
            // a dialog whose content changed keeps its element, and so its id.
            let appeared = !known.contains(&scope.id());
            self.routing.open_scope(scope.clone());
            if appeared {
                self.enter_focus_trap(scope);
            }
        }
        self.routing
            .set_declared_traps(declared.iter().map(FocusScope::id).collect());
    }

    #[doc = include_str!("focus.doc/enter_focus_trap.md")]
    fn enter_focus_trap(&mut self, scope: &FocusScope) {
        self.routing.save_focus_for_scope(self.focused);
        if self.focused.is_some_and(|node| scope.contains(node)) {
            return;
        }
        let Some(first) = FocusOrder::of(&self.tree, Some(scope)).step(None, false) else {
            return;
        };
        self.move_focus_to(Some(first), UiEvent::detached(EventOrigin::Synthetic));
    }

    #[doc = include_str!("focus.doc/leave_focus_trap.md")]
    fn leave_focus_trap(&mut self, id: FocusScopeId) {
        let closing = self.routing.scope(id).cloned();
        self.routing.close_scope(id);
        let saved = self.routing.restore_focus_for_scope();
        let held_focus = self.focused.is_none_or(|node| {
            self.tree.get(node).is_none()
                || closing.as_ref().is_some_and(|scope| scope.contains(node))
        });
        if !held_focus {
            return;
        }
        let target = saved.filter(|node| self.tree.get(*node).is_some());
        self.move_focus_to(target, UiEvent::detached(EventOrigin::Synthetic));
    }

    #[doc = include_str!("focus.doc/drop_focus_if_removed.md")]
    pub(super) fn drop_focus_if_removed(&mut self) {
        // Before the orphan check, not after: a trap that closed this frame took
        // the focused node with it, and the restore below is the answer to where
        // focus goes. Reversing the two would clear focus first and then restore
        // it, announcing a `blur` to nobody and a `focus` the user never lost.
        self.reconcile_focus_traps();
        let Some(node) = self.focused else {
            return;
        };
        if self.tree.get(node).is_some() {
            return;
        }
        // No `blur` reaches anyone: `binding_for` starts at a node the tree no
        // longer has, finds nothing, and reports nothing. That is the honest
        // outcome rather than a missing one — the node whose handler would run
        // is gone, and synthesising an ancestor to run it on would be inventing
        // a target the author never bound.
        self.move_focus_to(None, UiEvent::detached(EventOrigin::Synthetic));
    }

    #[doc = include_str!("focus.doc/dispatch_focus_change.md")]
    fn dispatch_focus_change(
        &mut self,
        blurred: Option<NodeId>,
        focused: Option<NodeId>,
        cause: UiEvent,
    ) {
        // Blur before focus, and both after the move is seated. Either half may
        // be absent: focusing the first control blurs nothing, and a press on
        // empty space focuses nothing.
        if let Some(node) = blurred {
            self.dispatch_focus_notification(node, EVENT_BLUR, cause);
        }
        if let Some(node) = focused {
            self.dispatch_focus_notification(node, EVENT_FOCUS, cause);
        }
    }

    #[doc = include_str!("focus.doc/dispatch_focus_notification.md")]
    fn dispatch_focus_notification(&mut self, node: NodeId, name: InternedSymbol, cause: UiEvent) {
        // Resolved through the ancestor chain, the same way `dispatch_raw_key`
        // resolves `keydown` on the same focused node — and NOT the way
        // `dispatch_crossing` resolves a hover crossing.
        //
        // The DOM's `focus` / `blur` do not bubble, and the reason that costs it
        // nothing is that the DOM focuses *elements*. This runtime focuses
        // whatever the press hit, and a press on the glyphs of a field hits the
        // field's text leaf: a rule that fired only for the exact focused node
        // would make `<input on:focus="…">` run or not depending on whether the
        // user clicked the text or the padding beside it. So the binding walks,
        // one node gains focus so the walk delivers exactly once, and
        // [`EVENT_FOCUS`](obscur_view::view::EVENT_FOCUS) states the deviation.
        let Some((node, command)) = CommandDispatch::binding_for(&self.tree, node, name) else {
            return;
        };
        // The outcome is deliberately dropped: focus has already moved, so there
        // is no default action left for `prevent_default` to suppress. The
        // refusal that exists is on `pointerdown`, one event earlier — see
        // [`EVENT_FOCUS`](obscur_view::view::EVENT_FOCUS).
        self.dispatch_command_outcome(Some(node), command, cause);
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Size;
    use obscur_identity::{ComponentId, RuntimeId};
    use obscur_interaction::focus::FOCUS_TRAP;
    use obscur_interaction::pointer::PointerEventKind;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
    use obscur_style::property::{Length, PropertyId, PropertyValue};
    use obscur_testkit::fixture::{ClassRule, mouse};
    use obscur_view::view::{
        CLASS, LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    use super::{EventOrigin, UiEvent};
    use crate::UiRuntime;
    use obscur_identity::NodeId;

    const PANEL: ComponentId = ComponentId::from_raw(0x00F0_0001);
    const BUTTON: ComponentId = ComponentId::from_raw(0x00F0_0002);
    const DIALOG: ComponentId = ComponentId::from_raw(0x00F0_0003);

    /// A tabbable control, classed so the sheet below can give it a box a press
    /// can land in.
    fn button(class: &str) -> ViewNode {
        ViewNode::element(BUTTON)
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("button".into()),
            ))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
    }

    /// A dialog element holding two controls. `modal` is carried in the reserved
    /// [`FOCUS_TRAP`] property exactly as `obscur_widgets::Dialog` emits it — the
    /// widget crate is not a dependency of this one, so the property is what the
    /// two agree on.
    fn dialog(modal: bool) -> ViewNode {
        dialog_holding(modal, vec![button("ok"), button("cancel")])
    }

    /// The same dialog element with content of the caller's choosing, so a
    /// dialog can hold another one.
    fn dialog_holding(modal: bool, content: Vec<ViewNode>) -> ViewNode {
        ViewNode::element(DIALOG)
            .with_prop(ViewProperty::new(FOCUS_TRAP, ViewValue::Bool(modal)))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("dialog".into())))
            .with_children(ViewFragment::from_nodes(content))
    }

    /// A page whose one control sits at y ∈ [0, 50), optionally followed by a
    /// dialog at y ∈ [50, 150) whose own two controls stack inside it.
    fn page(with_dialog: Option<bool>) -> View {
        let mut children = vec![button("page")];
        children.extend(with_dialog.map(dialog));
        View::single(ViewNode::element(PANEL).with_children(ViewFragment::from_nodes(children)))
    }

    fn boxed(class: &str, height: f32) -> StyleRule {
        ClassRule::new(class)
            .declaring(vec![
                BlockDeclaration::normal(
                    PropertyId::Width,
                    PropertyValue::Length(Length::px(100.0)),
                ),
                BlockDeclaration::normal(
                    PropertyId::Height,
                    PropertyValue::Length(Length::px(height)),
                ),
            ])
            .build()
    }

    fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                boxed("page", 50.0),
                boxed("dialog", 100.0),
                boxed("ok", 50.0),
                boxed("cancel", 50.0),
            ],
        )
    }

    fn runtime() -> UiRuntime {
        UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0))
    }

    /// `(page control, dialog's first control, dialog's second control)`.
    fn controls(rt: &UiRuntime) -> (NodeId, NodeId, NodeId) {
        let root = rt.tree().root().expect("a root");
        let panel = rt.tree().get(root).expect("live").child_nodes()[0];
        let kids = rt.tree().get(panel).expect("live").child_nodes().to_vec();
        let dialog = rt.tree().get(kids[1]).expect("live").child_nodes().to_vec();
        (kids[0], dialog[0], dialog[1])
    }

    /// The defect: Tab walked out of an open modal into the page behind it,
    /// because the trap was a type nothing opened. The dialog element declares
    /// it, so committing the dialog opens it and Tab cycles inside.
    #[test]
    fn tab_from_the_last_control_in_an_open_modal_returns_to_its_first() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(Some(true)), &sheets);
        let (page_control, ok, cancel) = controls(&rt);

        assert_eq!(
            rt.focused(),
            Some(ok),
            "opening the trap takes focus to its first stop"
        );
        assert!(rt.move_focus_by_tab(false, cause()));
        assert_eq!(rt.focused(), Some(cancel));
        assert!(rt.move_focus_by_tab(false, cause()));
        assert_eq!(
            rt.focused(),
            Some(ok),
            "the ring wraps inside the modal rather than reaching {page_control:?}"
        );
        assert!(rt.move_focus_by_tab(true, cause()));
        assert_eq!(
            rt.focused(),
            Some(cancel),
            "and Shift+Tab wraps back the same way"
        );
    }

    /// The control: with the same view declaring itself non-modal, the same Tab
    /// reaches the page. Without this the test above would pass on a runtime
    /// that simply refused to move focus at all.
    #[test]
    fn tab_leaves_a_dialog_that_is_not_modal() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(Some(false)), &sheets);
        let (page_control, _ok, cancel) = controls(&rt);

        assert!(rt.focus_node(Some(cancel)));
        assert!(rt.move_focus_by_tab(false, cause()));
        assert_eq!(
            rt.focused(),
            Some(page_control),
            "a non-modal dialog owns no ring"
        );
    }

    /// The defect: a press consulted no scope, so clicking the page behind an
    /// open modal moved focus straight out of it.
    #[test]
    fn a_press_outside_an_open_modal_does_not_focus_what_it_hit() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(Some(true)), &sheets);
        let (_page_control, ok, _cancel) = controls(&rt);

        rt.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(
            rt.focused(),
            Some(ok),
            "the press on the page behind the modal moved nothing"
        );
    }

    /// The control for the press: the identical press at the identical point
    /// focuses the page control when the dialog does not trap, so the assertion
    /// above is about the trap and not about a press that missed.
    #[test]
    fn the_same_press_focuses_the_page_behind_a_dialog_that_is_not_modal() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(Some(false)), &sheets);
        let (page_control, _ok, _cancel) = controls(&rt);

        rt.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
        assert_eq!(rt.focused(), Some(page_control));
    }

    /// The defect: `FocusRestore` had no references anywhere, so dismissing a
    /// dialog left focus nowhere and the next Tab started from the top.
    #[test]
    fn closing_a_modal_returns_focus_to_the_control_that_opened_it() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(None), &sheets);
        let root = rt.tree().root().expect("a root");
        let panel = rt.tree().get(root).expect("live").child_nodes()[0];
        let opener = rt.tree().get(panel).expect("live").child_nodes()[0];
        assert!(rt.focus_node(Some(opener)), "the control that opens it");

        let _ = rt.render_now(&page(Some(true)), &sheets);
        let (_page_control, ok, _cancel) = controls(&rt);
        assert_eq!(rt.focused(), Some(ok), "the trap took focus");

        let _ = rt.render_now(&page(None), &sheets);
        assert_eq!(
            rt.focused(),
            Some(opener),
            "and gave it back to the control that had it"
        );
    }

    /// A trap that opens over an unfocused surface must still push a frame, or
    /// its close pops a node that belongs to nobody. Here the close must simply
    /// leave focus nowhere rather than inventing a target.
    #[test]
    fn a_modal_opened_over_nothing_restores_nothing() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(None), &sheets);
        assert_eq!(rt.focused(), None, "nothing focused to begin with");

        let _ = rt.render_now(&page(Some(true)), &sheets);
        let (_page_control, ok, _cancel) = controls(&rt);
        assert_eq!(rt.focused(), Some(ok));

        let _ = rt.render_now(&page(None), &sheets);
        assert_eq!(rt.focused(), None);
    }

    /// Nested traps are why the saved focus is a stack and why the declaring
    /// nodes are ordered by depth: a dialog opened from a dialog must own the
    /// ring while it is up, and closing it must hand focus back one level rather
    /// than all the way out to the page.
    #[test]
    fn a_dialog_opened_from_a_dialog_owns_the_ring_and_gives_it_back_one_level() {
        let outer_only = View::single(ViewNode::element(PANEL).with_children(
            ViewFragment::from_nodes(vec![
                button("page"),
                dialog_holding(true, vec![button("ok")]),
            ]),
        ));
        let both = View::single(
            ViewNode::element(PANEL).with_children(ViewFragment::from_nodes(vec![
                button("page"),
                dialog_holding(
                    true,
                    vec![button("ok"), dialog_holding(true, vec![button("cancel")])],
                ),
            ])),
        );
        let mut rt = runtime();
        let sheets = [sheet()];

        let _ = rt.render_now(&outer_only, &sheets);
        let outer_control = rt.focused().expect("the outer trap took focus");

        let _ = rt.render_now(&both, &sheets);
        let inner_control = rt.focused().expect("the inner trap took it in turn");
        assert_ne!(inner_control, outer_control, "the inner one owns the ring");
        assert!(!rt.move_focus_by_tab(false, cause()));
        assert_eq!(
            rt.focused(),
            Some(inner_control),
            "and Tab has nowhere else to go inside it"
        );
        assert!(
            !rt.focus_node(Some(outer_control)),
            "the outer trap's own control is outside the inner one"
        );

        let _ = rt.render_now(&outer_only, &sheets);
        assert_eq!(
            rt.focused(),
            Some(outer_control),
            "closing the inner one restores one level, not all the way out"
        );
    }

    /// The third escape: a programmatic move honoured no scope either, so an
    /// application could hand focus to the page behind its own modal.
    #[test]
    fn a_programmatic_focus_outside_an_open_modal_is_refused() {
        let mut rt = runtime();
        let sheets = [sheet()];
        let _ = rt.render_now(&page(Some(true)), &sheets);
        let (page_control, ok, _cancel) = controls(&rt);

        assert!(!rt.focus_node(Some(page_control)), "refused");
        assert_eq!(rt.focused(), Some(ok));
        assert!(!rt.focus_node(None), "and so is clearing focus outright");
        assert_eq!(rt.focused(), Some(ok));
    }

    fn cause() -> UiEvent {
        UiEvent::detached(EventOrigin::Keyboard)
    }
}

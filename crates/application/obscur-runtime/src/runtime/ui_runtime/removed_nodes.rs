#![doc = include_str!("../../../docs/api/runtime/ui_runtime/removed_nodes.doc/_module.md")]

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/removed_nodes.doc/drop_state_of_removed_nodes.md")]
    pub(super) fn drop_state_of_removed_nodes(&mut self) {
        // The layout side has needed none of this: `arrange` replaces its whole
        // map from the pass output, so it is the live tree by construction. The
        // style side retains on purpose — that is what makes it incremental —
        // and retention is only ever half a policy until something states what
        // leaves.
        self.computed.retain(|node, _| self.tree.contains(*node));
        self.touch_gestures
            .retain(|_, candidate| self.tree.contains(candidate.bound));
        if self
            .system_back
            .is_some_and(|session| !self.tree.contains(session.bound))
        {
            self.system_back = None;
        }
        self.retain_live_indirect_gestures();
        self.inputs.retain_live(&self.tree);
        self.pseudo.retain_live(&self.tree);
        self.scroll.retain_live(&self.tree);
        self.scrollbar_drags
            .retain(|_, drag| self.tree.contains(drag.node));
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Size;
    use obscur_identity::{ComponentId, RuntimeId};
    use obscur_style::selector::PseudoState;
    use obscur_testkit::fixture::ScrollingList;
    use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

    use super::UiRuntime;

    /// Repeated mounting and removal keeps pseudo-class and scroll state bounded by live nodes.
    #[test]
    fn reconciliation_discards_removed_pseudo_states_and_scroll_offsets() {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0));
        let container = |id| {
            ViewNode::element(ComponentId::from_raw(id))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
        };
        let sheets = [ScrollingList::sheet()];
        let first = container(1);
        let second = container(2);
        let one = View::from_fragment(ViewFragment::from_nodes(vec![first.clone()]));
        let two = View::from_fragment(ViewFragment::from_nodes(vec![first, second]));
        let _ = runtime.render_now(&one, &sheets);
        let root = runtime.tree.root().unwrap();
        let survivor = runtime.tree.get(root).unwrap().child_nodes()[0];
        runtime.set_pseudo_state(survivor, PseudoState::HOVER);
        runtime.scroll.seat(survivor, 10.0);

        for _ in 0..10_000 {
            let _ = runtime.render_now(&two, &sheets);
            let removed = runtime.tree.get(root).unwrap().child_nodes()[1];
            runtime.set_pseudo_state(removed, PseudoState::ACTIVE);
            runtime.scroll.seat(removed, 20.0);
            assert_eq!(runtime.scroll.stored_offset_count(), 2);

            let _ = runtime.render_now(&one, &sheets);
            assert!(!runtime.tree.contains(removed));
            assert!(runtime.pseudo_state(removed).is_empty());
            assert_eq!(runtime.pseudo_state(survivor), PseudoState::HOVER);
            assert_eq!(runtime.scroll.stored_offset_count(), 1);
        }
    }
}

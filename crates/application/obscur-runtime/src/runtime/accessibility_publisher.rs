#![doc = include_str!("accessibility_publisher.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_accessibility::event::AccessibilityFocus;
use obscur_accessibility::port::AccessibilityBridge;
use obscur_accessibility::tree::SemanticSnapshot;
use obscur_identity::NodeId;

#[doc = include_str!("accessibility_publisher.doc/AccessibilityPublisher.md")]
pub(crate) struct AccessibilityPublisher {
    #[doc = include_str!("accessibility_publisher.doc/bridge.md")]
    bridge: Option<Rc<RefCell<dyn AccessibilityBridge>>>,
    #[doc = include_str!("accessibility_publisher.doc/published.md")]
    published: Option<NodeId>,
}

impl AccessibilityPublisher {
    #[doc = include_str!("accessibility_publisher.doc/new.md")]
    pub(crate) const fn new() -> Self {
        Self {
            bridge: None,
            published: None,
        }
    }

    #[doc = include_str!("accessibility_publisher.doc/install.md")]
    pub(crate) fn install(&mut self, bridge: Rc<RefCell<dyn AccessibilityBridge>>) {
        self.bridge = Some(bridge);
    }

    #[doc = include_str!("accessibility_publisher.doc/is_installed.md")]
    pub(crate) const fn is_installed(&self) -> bool {
        self.bridge.is_some()
    }

    #[doc = include_str!("accessibility_publisher.doc/installed.md")]
    pub(crate) fn installed(&self) -> Option<Rc<RefCell<dyn AccessibilityBridge>>> {
        self.bridge.clone()
    }

    #[doc = include_str!("accessibility_publisher.doc/should_publish.md")]
    pub(crate) fn should_publish(&self, tree_changed: bool, focused: Option<NodeId>) -> bool {
        self.is_installed() && (tree_changed || self.published != focused)
    }

    #[doc = include_str!("accessibility_publisher.doc/push.md")]
    pub(crate) fn push(
        &mut self,
        bridge: &mut dyn AccessibilityBridge,
        focused: Option<NodeId>,
        snapshot: &SemanticSnapshot,
    ) {
        bridge.set_focus(&AccessibilityFocus::new(focused, self.published));
        bridge.update_tree(snapshot);
        self.published = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::AccessibilityPublisher;
    use obscur_accessibility::port::{AccessibilityBridge, RecordingAccessibilityBridge};
    use obscur_accessibility::semantic::{SemanticNode, SemanticRole};
    use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
    use obscur_identity::NodeId;
    use std::cell::RefCell;
    use std::rc::Rc;

    const A: NodeId = NodeId::from_raw(1);
    const B: NodeId = NodeId::from_raw(2);

    fn snapshot() -> SemanticSnapshot {
        let mut tree = SemanticTree::new();
        tree.insert(SemanticNode::new(A, SemanticRole::Button), None);
        SemanticSnapshot::from_tree(&tree)
    }

    fn bridge() -> Rc<RefCell<RecordingAccessibilityBridge>> {
        Rc::new(RefCell::new(RecordingAccessibilityBridge::new()))
    }

    /// One frame, as the runtime performs it: ask the gate, and build-and-push
    /// only past it. Returns whether it published.
    fn frame(
        publisher: &mut AccessibilityPublisher,
        recorder: &Rc<RefCell<RecordingAccessibilityBridge>>,
        tree_changed: bool,
        focused: Option<NodeId>,
    ) -> bool {
        if !publisher.should_publish(tree_changed, focused) {
            return false;
        }
        publisher.push(&mut *recorder.borrow_mut(), focused, &snapshot());
        true
    }

    #[test]
    fn with_no_bridge_installed_nothing_is_ever_published() {
        let publisher = AccessibilityPublisher::new();
        assert!(!publisher.is_installed());
        assert!(publisher.installed().is_none());
        assert!(
            !publisher.should_publish(true, Some(A)),
            "the honest-capability default: no bridge, so no semantic tree is built at all"
        );
        assert!(!publisher.should_publish(false, None));
    }

    #[test]
    fn a_changed_tree_publishes_even_when_focus_stood_still() {
        let mut publisher = AccessibilityPublisher::new();
        let recorder = bridge();
        publisher.install(recorder.clone());
        assert!(frame(&mut publisher, &recorder, true, Some(A)));
        assert!(
            frame(&mut publisher, &recorder, true, Some(A)),
            "the tree is what changed; focus being settled does not gate it"
        );
        assert_eq!(recorder.borrow().tree_updates, 2);
    }

    #[test]
    fn a_focus_move_publishes_once_and_the_frames_after_it_publish_nothing() {
        // The pairing itself, and the direction that is expensive rather than
        // wrong: the push records what it published, so the gate closes behind
        // it. A push that failed to record would re-walk the whole retained tree
        // on every frame that followed, forever, with nothing to show for it.
        let mut publisher = AccessibilityPublisher::new();
        let recorder = bridge();
        publisher.install(recorder.clone());

        assert!(frame(&mut publisher, &recorder, false, Some(A)));
        for _ in 0..3 {
            assert!(
                !frame(&mut publisher, &recorder, false, Some(A)),
                "an idle, hovering or animating frame re-publishes nothing"
            );
        }
        assert!(frame(&mut publisher, &recorder, false, Some(B)));
        assert_eq!(recorder.borrow().tree_updates, 2);
    }

    #[test]
    fn each_push_reports_the_node_the_one_before_it_moved_to() {
        let mut publisher = AccessibilityPublisher::new();
        let recorder = bridge();
        publisher.install(recorder.clone());

        assert!(frame(&mut publisher, &recorder, false, Some(A)));
        let first = recorder.borrow().last_focus.expect("focus was reflected");
        assert_eq!(first.focused(), Some(A));
        assert_eq!(
            first.previous(),
            None,
            "nothing had been published, so nothing was focused before"
        );

        assert!(frame(&mut publisher, &recorder, false, Some(B)));
        let second = recorder.borrow().last_focus.expect("focus was reflected");
        assert_eq!(second.focused(), Some(B));
        assert_eq!(
            second.previous(),
            Some(A),
            "the node focus actually moved from, which is the record's whole job"
        );
        assert!(second.changed());
    }

    #[test]
    fn clearing_focus_publishes_and_names_the_node_it_was_cleared_from() {
        let mut publisher = AccessibilityPublisher::new();
        let recorder = bridge();
        publisher.install(recorder.clone());
        assert!(frame(&mut publisher, &recorder, false, Some(A)));
        assert!(
            frame(&mut publisher, &recorder, false, None),
            "Some -> None is a focus move like any other"
        );
        let cleared = recorder.borrow().last_focus.expect("focus was reflected");
        assert!(cleared.is_cleared());
        assert_eq!(cleared.previous(), Some(A));
        assert!(
            !frame(&mut publisher, &recorder, false, None),
            "and cleared stays published, so the next idle frame is gated"
        );
    }

    #[test]
    fn ordering_within_a_push_is_focus_then_tree() {
        // Stated by a test because an adapter converts a tree against the focus
        // target it was last told: a snapshot pushed before the focus arrives is
        // converted against the platform's stale target.
        #[derive(Default)]
        struct OrderRecorder(Vec<&'static str>);
        impl AccessibilityBridge for OrderRecorder {
            fn update_tree(&mut self, _snapshot: &SemanticSnapshot) {
                self.0.push("tree");
            }
            fn apply_update(&mut self, _update: &obscur_accessibility::tree::SemanticUpdate) {}
            fn set_focus(&mut self, _focus: &obscur_accessibility::event::AccessibilityFocus) {
                self.0.push("focus");
            }
            fn announce(&mut self, _message: &str, _assertive: bool) {}
            fn poll_actions(&mut self) -> Vec<obscur_accessibility::event::AccessibilityAction> {
                Vec::new()
            }
        }

        let mut publisher = AccessibilityPublisher::new();
        let mut order = OrderRecorder::default();
        publisher.push(&mut order, Some(A), &snapshot());
        assert_eq!(order.0, vec!["focus", "tree"]);
    }

    #[test]
    fn a_host_pushing_its_own_bridge_advances_the_installed_ones_record() {
        // Named as behaviour, not endorsed. There is one record, so the two push
        // routes are alternatives rather than parallel outputs: a host that
        // drives its own bridge *as well as* installing one satisfies the gate
        // with its own push, and the installed bridge misses that focus move
        // unless the tree changed too.
        let mut publisher = AccessibilityPublisher::new();
        let installed = bridge();
        publisher.install(installed.clone());

        let mut host_owned = RecordingAccessibilityBridge::new();
        publisher.push(&mut host_owned, Some(A), &snapshot());
        assert_eq!(host_owned.tree_updates, 1);

        assert!(
            !frame(&mut publisher, &installed, false, Some(A)),
            "the host's push already claimed the move"
        );
        assert_eq!(installed.borrow().tree_updates, 0);
    }

    #[test]
    fn installing_a_second_bridge_keeps_the_first_ones_record() {
        // The consequence of the record belonging to "whatever was last told"
        // rather than to a particular bridge, preserved rather than corrected —
        // see `install`.
        let mut publisher = AccessibilityPublisher::new();
        let first = bridge();
        publisher.install(first.clone());
        assert!(frame(&mut publisher, &first, false, Some(A)));

        let second = bridge();
        publisher.install(second.clone());
        publisher.push(&mut *second.borrow_mut(), Some(A), &snapshot());
        let reported = second.borrow().last_focus.expect("focus was reflected");
        assert_eq!(reported.focused(), Some(A));
        assert_eq!(
            reported.previous(),
            Some(A),
            "the previous node is the one the *first* bridge was told about"
        );
        assert!(
            !reported.changed(),
            "so a bridge that gates on `changed` learns nothing from its first push"
        );
    }
}

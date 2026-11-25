#![doc = include_str!("semantic_tree.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_accessibility::event::{AccessibilityAction, AccessibilityFocus};
use obscur_accessibility::port::{AccessibilityBridge, RecordingAccessibilityBridge};
use obscur_accessibility::semantic::{SemanticNode, SemanticRole};
use obscur_accessibility::tree::{SemanticSnapshot, SemanticUpdate};
use obscur_identity::ComponentId;
use obscur_runtime::UiRuntime;
use obscur_testkit::fixture::{ButtonElement, CounterPanel, TestFixture};
use obscur_view::view::{LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A bridge that keeps what it was handed. Counting pushes would let a test
/// pass while the runtime published an empty or stale tree, so this retains
/// the snapshot itself and the tests assert what is *in* it.
#[derive(Default)]
struct CapturingBridge {
    last: Option<SemanticSnapshot>,
    focus: Option<AccessibilityFocus>,
    pushes: u32,
}

impl AccessibilityBridge for CapturingBridge {
    fn update_tree(&mut self, snapshot: &SemanticSnapshot) {
        self.last = Some(snapshot.clone());
        self.pushes += 1;
    }
    fn apply_update(&mut self, _update: &SemanticUpdate) {}
    fn set_focus(&mut self, focus: &AccessibilityFocus) {
        self.focus = Some(*focus);
    }
    fn announce(&mut self, _message: &str, _assertive: bool) {}
    fn poll_actions(&mut self) -> Vec<AccessibilityAction> {
        Vec::new()
    }
}

/// Installs a capturing bridge and returns the shared handle a host would
/// keep.
fn install_capture(runtime: &mut UiRuntime) -> Rc<RefCell<CapturingBridge>> {
    let bridge = Rc::new(RefCell::new(CapturingBridge::default()));
    runtime.set_accessibility_bridge(bridge.clone());
    bridge
}

/// The node the semantic snapshot gave `role`, if exactly one has it.
fn semantic_with_role(snapshot: &SemanticSnapshot, role: SemanticRole) -> Option<SemanticNode> {
    snapshot
        .iter()
        .find(|(_, node)| node.role() == role)
        .map(|(_, node)| node.clone())
}

#[test]
fn semantic_snapshot_reflects_the_live_rendered_tree() {
    // A `button` element (CSS type) carrying a "Save" text child.
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(20))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("button".into()),
            ))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")])),
    ]));

    let mut runtime = TestFixture::runtime();
    let _ = runtime.render_now(&view, &[]);

    let snapshot = runtime.semantic_snapshot();
    // root + button element + text child.
    assert_eq!(snapshot.len(), 3);
    let root = runtime.tree().root().expect("root after a frame");
    assert_eq!(snapshot.root(), Some(root));

    let button = runtime.tree().get(root).unwrap().child_nodes()[0];
    let button_node = snapshot.get(button).expect("button in the snapshot");
    assert_eq!(button_node.role(), SemanticRole::Button);
    assert_eq!(button_node.name(), Some("Save"));

    // The text child maps to a static-text node named by its content.
    let text_child = runtime.tree().get(button).unwrap().child_nodes()[0];
    let text_node = snapshot.get(text_child).expect("text in the snapshot");
    assert_eq!(text_node.role(), SemanticRole::Text);
    assert_eq!(text_node.name(), Some("Save"));

    // Determinism: the same live tree yields an equal snapshot.
    assert_eq!(runtime.semantic_snapshot(), snapshot);
}

#[test]
fn a_frame_publishes_the_live_editable_value_and_directional_selection() {
    use obscur_accessibility::semantic::SemanticValue;
    use obscur_interaction::command::{
        KeyEvent, LogicalKey, Modifier, Modifiers, NamedKey, PhysicalKey,
    };

    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(20))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("input".into()),
            ))
            .with_event(obscur_view::view::ViewEventBinding::new(
                obscur_view::view::EVENT_INPUT,
                obscur_identity::InternedSymbol::named("edit"),
            ))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("aé")])),
    ]));
    let mut runtime = TestFixture::runtime();
    let bridge = install_capture(&mut runtime);
    let _ = runtime.render_now(&view, &[]);
    let root = runtime.tree().root().unwrap();
    let field = runtime.tree().get(root).unwrap().child_nodes()[0];
    assert!(runtime.focus_node(Some(field)));
    let _ = runtime.render_now(&view, &[]);
    runtime.set_modifiers(Modifiers::new().with(Modifier::Shift));
    let _ = runtime.handle_key(KeyEvent::press(
        PhysicalKey::ArrowLeft,
        Some(LogicalKey::Named(NamedKey::ArrowLeft)),
    ));
    let _ = runtime.run_due_frame(&view, &[]);

    let seen = bridge.borrow();
    let node = seen.last.as_ref().unwrap().get(field).unwrap();
    assert_eq!(node.value(), &SemanticValue::text("aé"));
    assert_eq!(node.text_selection, Some([3, 1]));
}

#[test]
fn push_accessibility_feeds_the_bridge() {
    let mut runtime = TestFixture::runtime();
    let _ = runtime.render_now(&CounterPanel::view(), &[]);

    let mut bridge = RecordingAccessibilityBridge::new();
    runtime.push_accessibility(&mut bridge);
    assert_eq!(bridge.tree_updates, 1, "the snapshot reached the bridge");
}

/// The §7-A residue: the builder derives the truth, but until the frame path
/// pushed it, the only caller was a test. An ordinary frame — nobody calling
/// `push_accessibility` — must land the tree on the installed bridge.
#[test]
fn an_ordinary_frame_publishes_the_semantic_tree_to_the_installed_bridge() {
    let mut runtime = TestFixture::runtime();
    let bridge = install_capture(&mut runtime);

    let _ = runtime.render_now(&ButtonElement::state_view(true), &[]);

    let seen = bridge.borrow();
    let snapshot = seen.last.as_ref().expect("a frame published a snapshot");
    assert!(
        snapshot.root().is_some(),
        "the frame's own tree, not the empty one the install pushed"
    );
    let button =
        semantic_with_role(snapshot, SemanticRole::Button).expect("the button was published");
    assert_eq!(button.name(), Some("Save"));
    assert!(
        button.state().disabled,
        "the state §7-A fixed in the builder is what reached the bridge"
    );
    assert!(button.actions().is_empty());
}

/// The push sits on the frame path, and the semantic tree is a full walk of
/// the retained tree — so a frame that changed neither the tree nor focus must
/// not rebuild and re-publish it.
#[test]
fn a_frame_that_changes_nothing_republishes_nothing() {
    let mut runtime = TestFixture::runtime();
    let bridge = install_capture(&mut runtime);
    let view = ButtonElement::state_view(false);
    let _ = runtime.render_now(&view, &[]);

    let settled = bridge.borrow().pushes;
    let _ = runtime.render_now(&view, &[]);
    let _ = runtime.render_now(&view, &[]);
    assert_eq!(
        bridge.borrow().pushes,
        settled,
        "an idempotent frame publishes nothing new"
    );
}

/// A host that installs a bridge after the UI is already on screen must not
/// have to wait for an unrelated change to publish what is already there.
#[test]
fn installing_a_bridge_publishes_the_tree_already_on_screen() {
    let mut runtime = TestFixture::runtime();
    assert!(!runtime.has_accessibility_bridge());
    let _ = runtime.render_now(&ButtonElement::state_view(false), &[]);

    let bridge = install_capture(&mut runtime);
    assert!(runtime.has_accessibility_bridge());
    let seen = bridge.borrow();
    let snapshot = seen.last.as_ref().expect("install published");
    assert!(semantic_with_role(snapshot, SemanticRole::Button).is_some());
}

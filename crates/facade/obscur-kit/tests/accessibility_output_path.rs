#![doc = include_str!("accessibility_output_path.doc/_module.md")]

use accesskit::{Action, NodeId as AkNodeId, Role, Toggled, TreeUpdate};
use obscur_accessibility_accesskit::bridge::AccessKitBridge;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::view::view::{
    CHECKED, CLASS, DISABLED, LOCAL_NAME, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

use std::cell::RefCell;
use std::rc::Rc;

fn runtime() -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0))
}

#[test]
fn accesskit_can_change_the_live_text_selection() {
    use accesskit::{ActionData, ActionRequest, TextPosition, TextSelection};
    use obscur_kit::identity::InternedSymbol;
    use obscur_kit::view::view::{EVENT_INPUT, ViewEventBinding};

    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        element(2, "textarea", "aé\nz", &[]).with_event(ViewEventBinding::new(
            EVENT_INPUT,
            InternedSymbol::named("edit"),
        )),
    ]));
    let mut runtime = runtime();
    let bridge = Rc::new(RefCell::new(AccessKitBridge::new()));
    runtime.set_accessibility_bridge(bridge.clone());
    let _ = runtime.render_now(&view, &[]);
    let root = runtime.tree().root().unwrap();
    let field = runtime.tree().get(root).unwrap().child_nodes()[0];
    assert!(runtime.focus_node(Some(field)));
    let _ = runtime.render_now(&view, &[]);
    let children = {
        let bridge = bridge.borrow();
        let update = bridge.last_tree_update().unwrap();
        let field = update
            .nodes
            .iter()
            .find(|(id, _)| *id == update.focus)
            .unwrap();
        assert!(field.1.supports_action(Action::SetTextSelection));
        field.1.children().to_vec()
    };
    let [first, second] = children[children.len() - 2..] else {
        unreachable!()
    };
    let request = |node, index| ActionRequest {
        action: Action::SetTextSelection,
        target: AkNodeId(field.raw()),
        data: Some(ActionData::SetTextSelection(TextSelection {
            anchor: TextPosition {
                node: second,
                character_index: 1,
            },
            focus: TextPosition {
                node,
                character_index: index,
            },
        })),
    };
    assert!(
        !bridge
            .borrow_mut()
            .push_action_request(request(AkNodeId(999), 1))
    );
    assert!(!bridge.borrow_mut().push_action_request(request(first, 99)));
    assert!(bridge.borrow_mut().push_action_request(request(first, 1)));
    assert_eq!(runtime.poll_accessibility_actions(), 1);
    let selection = runtime.editing_selection().unwrap();
    assert_eq!([selection.anchor(), selection.focus()], [5, 1]);
}

#[doc = include_str!("accessibility_output_path.doc/element.md")]
fn element(component: u64, local_name: &str, label: &str, state: &[ViewProperty]) -> ViewNode {
    let mut node = ViewNode::element(ComponentId::from_raw(component))
        .with_prop(ViewProperty::new(
            LOCAL_NAME,
            ViewValue::Text(local_name.into()),
        ))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(label)]));
    for prop in state {
        node = node.with_prop(prop.clone());
    }
    node
}

#[doc = include_str!("accessibility_output_path.doc/panel_view.md")]
fn panel_view() -> View {
    let disabled = ViewProperty::new(DISABLED, ViewValue::Bool(true));
    let checked = ViewProperty::new(CHECKED, ViewValue::Bool(true));
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(
                LOCAL_NAME,
                ViewValue::Text("region".into()),
            ))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                element(2, "button", "Save", std::slice::from_ref(&disabled)),
                element(3, "checkbox", "Agree", std::slice::from_ref(&checked)),
            ])),
    ]))
}

#[doc = include_str!("accessibility_output_path.doc/rendered_with_bridge.md")]
fn rendered_with_bridge() -> Rc<RefCell<AccessKitBridge>> {
    let mut runtime = runtime();
    let bridge = Rc::new(RefCell::new(AccessKitBridge::new()));
    runtime.set_accessibility_bridge(bridge.clone());
    let _ = runtime.render_now(&panel_view(), &[]);
    bridge
}

#[doc = include_str!("accessibility_output_path.doc/labelled.md")]
fn labelled(update: &TreeUpdate, label: &str) -> (AkNodeId, accesskit::Node) {
    update
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .cloned()
        .unwrap_or_else(|| panic!("no AccessKit node labelled {label:?}"))
}

#[doc = include_str!("accessibility_output_path.doc/an_ordinary_frame_lands_a_tree_update_on_the_accesskit_bridge.md")]
#[test]
fn an_ordinary_frame_lands_a_tree_update_on_the_accesskit_bridge() {
    let bridge = rendered_with_bridge();
    let bridge = bridge.borrow();
    let update = bridge
        .last_tree_update()
        .expect("the frame path converted and stored a TreeUpdate");

    assert!(update.tree.is_some(), "a rooted AccessKit tree");
    let (_, button) = labelled(update, "Save");
    assert_eq!(button.role(), Role::Button);
}

#[doc = include_str!("accessibility_output_path.doc/the_published_tree_carries_the_state_the_builder_derived.md")]
#[test]
fn the_published_tree_carries_the_state_the_builder_derived() {
    let bridge = rendered_with_bridge();
    let bridge = bridge.borrow();
    let update = bridge.last_tree_update().expect("stored");

    let (_, button) = labelled(update, "Save");
    assert!(
        button.is_disabled(),
        "a disabled button is announced disabled, not actionable"
    );
    assert!(
        !button.supports_action(Action::Click),
        "and advertises no click for an assistive technology to invoke"
    );

    let (_, checkbox) = labelled(update, "Agree");
    assert_eq!(checkbox.role(), Role::CheckBox);
    assert_eq!(checkbox.toggled(), Some(Toggled::True));
}

#[doc = include_str!("accessibility_output_path.doc/the_published_tree_keeps_the_retained_trees_nesting.md")]
#[test]
fn the_published_tree_keeps_the_retained_trees_nesting() {
    let bridge = rendered_with_bridge();
    let bridge = bridge.borrow();
    let update = bridge.last_tree_update().expect("stored");

    let root = update.tree.as_ref().expect("rooted").root;
    let panel_id = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Region)
        .map(|(id, _)| *id)
        .expect("the panel was published");
    let (button_id, _) = labelled(update, "Save");
    let (checkbox_id, _) = labelled(update, "Agree");

    let children = |id: AkNodeId| {
        update
            .nodes
            .iter()
            .find(|(nid, _)| *nid == id)
            .map(|(_, n)| n.children().to_vec())
            .unwrap_or_default()
    };

    assert_eq!(
        children(root),
        vec![panel_id],
        "the panel hangs off the root"
    );
    assert_eq!(
        children(panel_id),
        vec![button_id, checkbox_id],
        "the controls stay INSIDE the panel, in authored order, rather than being \
         flattened onto the root"
    );
}

#[doc = include_str!("accessibility_output_path.doc/a_controls_text_child_stays_its_child.md")]
#[test]
fn a_controls_text_child_stays_its_child() {
    let bridge = rendered_with_bridge();
    let bridge = bridge.borrow();
    let update = bridge.last_tree_update().expect("stored");

    let (button_id, _) = labelled(update, "Save");
    let text = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Label && n.label() == Some("Save"))
        .map(|(id, _)| *id);
    let button_children = update
        .nodes
        .iter()
        .find(|(id, _)| *id == button_id)
        .map(|(_, n)| n.children().to_vec())
        .unwrap_or_default();
    assert_eq!(
        button_children,
        text.into_iter().collect::<Vec<_>>(),
        "the button's text node is published beneath it"
    );
}

#[doc = include_str!("accessibility_output_path.doc/focus_reaches_the_bridge_with_the_tree.md")]
#[test]
fn focus_reaches_the_bridge_with_the_tree() {
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;
    use obscur_kit::css::source::CssSource;
    use obscur_kit::geometry::Point;
    use obscur_kit::interaction::pointer::{
        PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };

    // One focusable, hit-testable button.
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        element(2, "button", "Save", &[])
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into()))),
    ]));
    let src = CssSource::anonymous(".btn { width: 100px; height: 50px }");
    let sheet = StylesheetCompiler::new()
        .compile(&StylesheetParser::new(&src).parse())
        .matchable;

    let mut runtime = runtime();
    let bridge = Rc::new(RefCell::new(AccessKitBridge::new()));
    runtime.set_accessibility_bridge(bridge.clone());
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&view, sheets);
    assert_eq!(
        bridge.borrow().last_tree_update().expect("stored").focus,
        AkNodeId(runtime.tree().root().expect("rooted").raw()),
        "nothing is focused yet, so AccessKit is pointed at the root"
    );

    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Down,
        Point::new(50.0, 25.0),
    ));
    let focused = runtime.focused().expect("the press focused the button");
    let _ = runtime.render_now(&view, sheets);

    let bridge = bridge.borrow();
    let update = bridge.last_tree_update().expect("stored");
    assert_eq!(
        update.focus,
        AkNodeId(focused.raw()),
        "the platform is told which node has focus, not just that the tree changed"
    );
}

#[test]
fn platform_actions_reach_the_runtime_and_ignore_disabled_or_stale_targets() {
    use accesskit::ActionRequest;
    use obscur_kit::identity::{InternedSymbol, NodeId};
    use obscur_kit::interaction::event::{EventOrigin, UiEvent};
    use obscur_kit::view::view::{EVENT_CLICK, ViewEventBinding};
    use std::cell::Cell;

    let mut runtime = runtime();
    let bridge = Rc::new(RefCell::new(AccessKitBridge::new()));
    runtime.set_accessibility_bridge(bridge.clone());

    let activate = InternedSymbol::named("a11y_activate");
    let count = Rc::new(Cell::new(0));
    let activated = Rc::clone(&count);
    runtime.on_command(activate, move |event: &mut UiEvent| {
        assert_eq!(event.origin(), EventOrigin::Accessibility);
        assert!(event.is_trusted());
        activated.set(activated.get() + 1);
    });
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        element(1, "button", "Run", &[]).with_event(ViewEventBinding::new(EVENT_CLICK, activate)),
        element(
            2,
            "button",
            "Disabled",
            &[ViewProperty::new(DISABLED, ViewValue::Bool(true))],
        )
        .with_event(ViewEventBinding::new(EVENT_CLICK, activate)),
    ]));
    let _ = runtime.render_now(&view, &[]);
    while runtime.run_due_frame(&view, &[]).is_some() {}

    let (target, disabled) = {
        let bridge = bridge.borrow();
        let snapshot = bridge.last_tree_update().unwrap();
        (
            labelled(snapshot, "Run").0,
            labelled(snapshot, "Disabled").0,
        )
    };
    let queue = |action, target| {
        bridge.borrow_mut().push_action_request(ActionRequest {
            action,
            target,
            data: None,
        })
    };

    assert!(!queue(Action::SetValue, target));
    assert!(queue(Action::Click, AkNodeId(u64::MAX)));
    assert!(queue(Action::Click, disabled));
    assert!(queue(Action::Focus, disabled));
    assert!(queue(Action::Focus, target));
    assert!(queue(Action::Click, target));

    assert!(runtime.run_due_frame(&view, &[]).is_some());
    assert_eq!(count.get(), 1);
    assert_eq!(runtime.focused(), Some(NodeId::from_raw(target.0)));
    assert_eq!(runtime.poll_accessibility_actions(), 0);
}

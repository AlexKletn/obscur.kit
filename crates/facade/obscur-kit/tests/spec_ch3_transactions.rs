#![doc = include_str!("spec_ch3_transactions.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch3_support/mod.rs"]
mod support;

use obscur_kit::runtime::transaction::{TransactionValidator, UiOperation, UiTransaction};
use support::{approx, at, drain_frames, el, geom, runtime, view};

#[test]
fn tree_expansion_restores_by_id_after_model_changes() {
    use obscur_kit::widgets::tree::{Tree, TreeExpansionState, TreeNode};

    let mut original = Tree::new().with_nodes([
        TreeNode::new("Files", 0, true).with_id("files"),
        TreeNode::new("Retired", 0, true).with_id("retired"),
        TreeNode::new("Reports", 0, true).with_id("reports"),
    ]);
    for index in 0..3 {
        assert!(original.expand(index));
    }
    let encoded = serde_json::to_string(&original.capture_expansion_state().unwrap()).unwrap();
    let state: TreeExpansionState = serde_json::from_str(&encoded).unwrap();

    let mut restored = Tree::new().with_nodes([
        TreeNode::new("Reports", 0, false).with_id("reports"),
        TreeNode::new("New", 0, true).with_id("new"),
        TreeNode::new("Файлы", 0, true).with_id("files"),
        TreeNode::new("Transient", 0, true),
    ]);
    assert!(restored.expand(1));
    assert!(restored.expand(3));
    assert!(restored.restore_expansion_state(&state));
    assert!(!restored.is_expanded(0));
    assert!(!restored.is_expanded(1));
    assert!(restored.is_expanded(2));
    assert!(restored.is_expanded(3));

    let before = restored.clone();
    let invalid =
        serde_json::from_str(r#"{"version":1,"expanded_ids":["files","files"]}"#).unwrap();
    assert!(!restored.restore_expansion_state(&invalid));
    assert_eq!(restored, before);
}

#[test]
fn docking_state_restores_known_panels_and_keeps_new_panels() {
    use obscur_kit::layout::algorithm::DockSide;
    use obscur_kit::widgets::docking::{DockPanel, Docking, DockingState};

    let original = Docking::new().with_panels([
        DockPanel::new("project", "Project", DockSide::Left),
        DockPanel::new("retired", "Retired", DockSide::Top),
        DockPanel::new("editor", "Editor", DockSide::Fill),
    ]);
    let encoded = serde_json::to_string(&original.capture_layout_state().unwrap()).unwrap();
    let state: DockingState = serde_json::from_str(&encoded).unwrap();
    let mut current = Docking::new().with_panels([
        DockPanel::new("editor", "Редактор", DockSide::Right),
        DockPanel::new("outline", "Outline", DockSide::Top),
        DockPanel::new("project", "Проект", DockSide::Bottom),
        DockPanel::new("logs", "Logs", DockSide::Bottom),
    ]);
    assert!(current.restore_layout_state(&state));
    assert_eq!(
        current
            .panels()
            .iter()
            .map(DockPanel::id)
            .collect::<Vec<_>>(),
        ["project", "editor", "outline", "logs"]
    );
    assert_eq!(current.panels()[0].side(), DockSide::Left);
    assert_eq!(current.panels()[1].title(), "Редактор");
    assert_eq!(current.panels()[2].side(), DockSide::Top);

    let before = current.clone();
    for document in [
        r#"{"version":2,"panels":[]}"#,
        r#"{"version":1,"panels":[["editor","left"],["project","unknown"]]}"#,
        r#"{"version":1,"panels":[["editor","left"],["editor","right"]]}"#,
    ] {
        let invalid = serde_json::from_str(document).unwrap();
        assert!(!current.restore_layout_state(&invalid));
        assert_eq!(current, before);
    }
}

#[test]
fn tab_selection_survives_reordering_and_label_changes() {
    use obscur_kit::widgets::tabs::{TabSelectionState, Tabs};

    let mut first = Tabs::new()
        .tab_with_id("files", "Files")
        .tab_with_id("settings", "Settings");
    assert!(first.activate(1));
    let encoded = serde_json::to_string(&first.capture_selection_state().unwrap()).unwrap();
    let state: TabSelectionState = serde_json::from_str(&encoded).unwrap();
    let mut restored = Tabs::new()
        .tab_with_id("settings", "Настройки")
        .tab_with_id("files", "Файлы");
    assert!(restored.activate(1));
    assert!(restored.restore_selection_state(&state));
    assert_eq!(restored.active_label(), Some("Настройки"));

    let mut ambiguous = Tabs::new()
        .tab("Overview")
        .tab_with_id("settings", "Settings A")
        .tab_with_id("settings", "Settings B");
    assert!(!ambiguous.restore_selection_state(&state));
    assert!(ambiguous.capture_selection_state().is_none());
    assert!(ambiguous.activate(1));
    assert!(ambiguous.capture_selection_state().is_none());

    let unknown = serde_json::from_str(r#"{"version":2,"tab_id":"files"}"#).unwrap();
    assert!(!restored.restore_selection_state(&unknown));
    assert_eq!(restored.active_label(), Some("Настройки"));
}

#[test]
fn focus_state_restores_only_a_unique_eligible_target() {
    use obscur_kit::runtime::FocusState;
    use obscur_kit::ui_macros::okui;

    let original_view = okui! { <div><button automation-id="submit">"Submit"</button></div> };
    let button_style = support::compile("button { display: block; width: 100px; height: 24px }");
    let sheets = std::slice::from_ref(&button_style);
    let mut original = runtime(300.0, 400.0);
    let _ = original.render_now(&original_view, sheets);
    assert!(original.focus_node(Some(at(&original, &[0, 0]))));
    let encoded = serde_json::to_string(&original.capture_focus_state().unwrap()).unwrap();
    let state: FocusState = serde_json::from_str(&encoded).unwrap();

    let next_view = okui! {
        <div><button>"Back"</button><button automation-id="submit">"Submit"</button></div>
    };
    let mut restored = runtime(300.0, 400.0);
    let _ = restored.render_now(&next_view, sheets);
    assert!(restored.restore_focus_state(&state));
    assert_eq!(restored.focused(), Some(at(&restored, &[0, 1])));

    let disabled_view = okui! { <div><button automation-id="submit" disabled/></div> };
    let duplicate_view = okui! {
        <div><button automation-id="submit"/><button automation-id="submit"/></div>
    };
    let hidden_view = okui! { <div><button class="hidden" automation-id="submit"/></div> };
    let hidden_style = support::compile(
        "button { display: block; width: 100px; height: 24px } .hidden { display: none }",
    );
    for (view, sheets) in [
        (&disabled_view, sheets),
        (&duplicate_view, sheets),
        (&hidden_view, std::slice::from_ref(&hidden_style)),
    ] {
        let mut runtime = runtime(300.0, 400.0);
        let _ = runtime.render_now(view, sheets);
        assert!(!runtime.restore_focus_state(&state));
        assert_eq!(runtime.focused(), None);
    }
}

#[test]
fn scroll_state_round_trips_between_runtimes_and_clamps_both_axes() {
    use obscur_kit::identity::NodeId;
    use obscur_kit::runtime::ScrollState;

    let first = support::compile(
        ".scroller { width: 300px; height: 400px; overflow: auto }
         .content { width: 900px; height: 1000px }",
    );
    let second = support::compile(
        ".scroller { width: 300px; height: 400px; overflow: auto }
         .content { width: 400px; height: 600px }",
    );
    let view = view(vec![el(1, "scroller", vec![el(2, "content", vec![])])]);
    let mut original = runtime(300.0, 400.0);
    let _ = original.render_now(&view, &[first]);
    let node = at(&original, &[0]);
    original.set_scroll_left(node, 350.0);
    original.set_scroll_top(node, 450.0);
    let snapshot = original.capture_scroll_state(node).unwrap();
    let encoded = serde_json::to_string(&snapshot).unwrap();
    let snapshot: ScrollState = serde_json::from_str(&encoded).unwrap();

    let mut restored = runtime(300.0, 400.0);
    let _ = restored.render_now(&view, &[second]);
    let node = at(&restored, &[0]);
    assert!(restored.restore_scroll_state(node, snapshot));
    approx(restored.scroll_left(node), 100.0, "horizontal clamp");
    approx(restored.scroll_top(node), 200.0, "vertical clamp");
    assert!(restored.has_pending_frame());
    assert!(!restored.restore_scroll_state(NodeId::from_raw(500), snapshot));

    let unknown = serde_json::from_str(r#"{"version":2,"x":0,"y":0}"#).unwrap();
    assert!(!restored.restore_scroll_state(node, unknown));
    approx(
        restored.scroll_top(node),
        200.0,
        "unknown version leaves state intact",
    );
}

#[doc = include_str!("spec_ch3_transactions.doc/a_batch_with_one_invalid_op_applies_none_of_it.md")]
#[test]
fn a_batch_with_one_invalid_op_applies_none_of_it() {
    const CSS: &str = ".page { width: 300px; height: 300px } \
                       .a { width: 100px; height: 20px }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "page", vec![el(2, "a", vec![])])]);

    let mut rt = runtime(300.0, 300.0);
    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let node_a = at(&rt, &[0, 0]);
    approx(
        geom(&rt, node_a).2,
        100.0,
        "node A's width before the commit",
    );

    let mut tx = UiTransaction::new();
    tx.push(UiOperation::InvalidateLayout { node: node_a }); // op1: valid
    tx.push(UiOperation::InvalidateLayout { node: node_a }); // op2: invalid — same node, same kind

    let outcome = TransactionValidator::new().validate(&tx);
    let error = outcome.expect_err(
        "the commit must return a typed validation failure — a batch is committed \
         atomically, so one bad op rejects the whole batch",
    );
    assert!(
        error.to_string().contains(&node_a.raw().to_string()),
        "the failure names op2's target ({}), got: {error}",
        node_a.raw()
    );

    // Nothing was applied and nothing was scheduled.
    assert!(
        !rt.has_pending_frame(),
        "a rejected commit requests no frame"
    );
    let _ = drain_frames(&mut rt, &v, sheets);
    let node_a = at(&rt, &[0, 0]);
    approx(
        geom(&rt, node_a).2,
        100.0,
        "node A keeps its PREVIOUS width — op1 must not have been applied",
    );
}

#[doc = include_str!("spec_ch3_transactions.doc/a_restored_scroll_offset_is_clamped_to_the_new_content_extent.md")]
#[test]
fn a_restored_scroll_offset_is_clamped_to_the_new_content_extent() {
    const CSS: &str = ".scroller { width: 300px; height: 400px; overflow: auto } \
                       .content { width: 300px; height: 600px }";

    let sheet = support::compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let v = view(vec![el(1, "scroller", vec![el(2, "content", vec![])])]);

    let mut rt = runtime(300.0, 400.0);
    let _ = rt.render_now(&v, sheets);
    let _ = drain_frames(&mut rt, &v, sheets);

    let scroller = at(&rt, &[0]);

    // The persisted offset, from a session whose content was 2000px tall.
    rt.set_scroll_top(scroller, 1500.0);
    let _ = drain_frames(&mut rt, &v, sheets);
    let _ = rt.render_now(&v, sheets);

    let scroller = at(&rt, &[0]);
    approx(
        rt.scroll_top(scroller),
        200.0,
        "the restored offset is clamped to this session's max scroll (600 - 400), \
         not written through raw and not lost to 0",
    );
}

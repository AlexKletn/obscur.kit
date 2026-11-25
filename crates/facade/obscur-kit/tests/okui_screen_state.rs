#![doc = include_str!("okui_screen_state.doc/_module.md")]
#![forbid(unsafe_code)]

use std::time::Instant;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::control::AppControl;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::headless::input::{SelectorResolver, SelectorTarget};
use obscur_kit::identity::{InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::okui_compiler::bridge::UiIrViewBridge;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::screen::{OkuiScreen, ScreenData, ScreenRow};
use obscur_kit::view::view::{TEST_ID, ViewValue};

#[doc = include_str!("okui_screen_state.doc/INCREMENT.md")]
const INCREMENT: InternedSymbol = InternedSymbol::named("increment");

#[doc = include_str!("okui_screen_state.doc/COUNTER.md")]
const COUNTER: &str = r#"<div class="counter">
    <span class="value" test-id="value">{count}</span>
    <div class="button" test-id="plus" on:click=increment/>
</div>"#;

const CSS: &str = ".counter { width: 200px; height: 100px } \
                   .value { width: 200px; height: 50px } \
                   .button { width: 200px; height: 50px }";

#[doc = include_str!("okui_screen_state.doc/host.md")]
fn host(screen: OkuiScreen) -> (UiRuntime, ScreenData, AppControl, CssPipeline) {
    let data = screen.data().clone();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let mut css = CssPipeline::new();
    let control = AppControl::new();
    control.set_stylesheet(CSS);
    control.apply(&mut runtime, &mut css);
    runtime.set_root_view(move || screen.view());
    (runtime, data, control, css)
}

#[doc = include_str!("okui_screen_state.doc/frame.md")]
fn frame(runtime: &mut UiRuntime) {
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("a requested frame is due");
}

#[doc = include_str!("okui_screen_state.doc/node.md")]
fn node(runtime: &UiRuntime, test_id: &str) -> NodeId {
    SelectorResolver::new()
        .resolve(runtime, &SelectorTarget::test_id(test_id))
        .unwrap_or_else(|| panic!("no node carries test-id=\"{test_id}\""))
}

#[doc = include_str!("okui_screen_state.doc/rendered_text.md")]
fn rendered_text(runtime: &UiRuntime, test_id: &str) -> String {
    let element = node(runtime, test_id);
    let children = runtime
        .tree()
        .get(element)
        .expect("the element is retained")
        .child_nodes()
        .to_vec();
    children
        .iter()
        .find_map(|child| {
            runtime
                .tree()
                .get(*child)
                .and_then(|node| node.kind().text().map(str::to_owned))
        })
        .expect("a text child")
}

#[doc = include_str!("okui_screen_state.doc/click.md")]
fn click(runtime: &mut UiRuntime, x: f32, y: f32) {
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        runtime.handle_pointer(
            PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                Point::new(x, y),
            )
            .as_trusted(),
        );
    }
}

#[doc = include_str!("okui_screen_state.doc/a_screen_reflects_changed_state_across_frames.md")]
#[test]
fn a_screen_reflects_changed_state_across_frames() {
    let screen = OkuiScreen::compile(COUNTER).expect("the screen compiles");
    let (mut runtime, data, _control, _css) = host(screen);
    data.set("count", 0_i64);

    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "value"), "0");

    data.set("count", 7_i64);
    frame(&mut runtime);
    assert_eq!(
        rendered_text(&runtime, "value"),
        "7",
        "the same compiled screen showed a new value — a cloned `View` could not"
    );
}

#[doc = include_str!("okui_screen_state.doc/a_click_authored_in_the_markup_changes_what_the_screen_shows.md")]
#[test]
fn a_click_authored_in_the_markup_changes_what_the_screen_shows() {
    let screen = OkuiScreen::compile(COUNTER).expect("the screen compiles");
    let (mut runtime, data, _control, _css) = host(screen);
    data.set("count", 0_i64);
    {
        // Exactly what `App::on_command` forwards: a handler that captures the
        // `ScreenData` handle. No runtime is touched, so no deferral is needed.
        let state = data.clone();
        runtime.on_command(INCREMENT, move || {
            state.set("count", state.int("count") + 1);
        });
    }
    frame(&mut runtime);

    // The button occupies y ∈ [50, 100) — below the 50px-tall value row.
    click(&mut runtime, 10.0, 75.0);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "value"), "1");

    click(&mut runtime, 10.0, 75.0);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "value"), "2");
}

#[doc = include_str!("okui_screen_state.doc/a_bound_class_reaches_the_cascade.md")]
#[test]
fn a_bound_class_reaches_the_cascade() {
    // The name is declared to the compiler by being bound before the compile, which is
    // also what makes a misspelled binding a §2.13 diagnostic at startup.
    let data = ScreenData::new();
    data.set("theme", "counter");
    let screen = OkuiScreen::compile_with(r#"<div class={theme} test-id="skin"/>"#, data.clone())
        .expect("the screen compiles");
    let (mut runtime, data, _control, _css) = host(screen);
    frame(&mut runtime);
    let height = |runtime: &UiRuntime| {
        runtime
            .layout_boxes()
            .get(&node(runtime, "skin"))
            .expect("laid out")
            .size()
            .height
    };
    assert_eq!(height(&runtime), 100.0, "`.counter` sized it");

    data.set("theme", "button");
    frame(&mut runtime);
    assert_eq!(
        height(&runtime),
        50.0,
        "the substituted class decided which rule matched — the value reached the \
         matcher, not just the tree"
    );
}

#[doc = include_str!("okui_screen_state.doc/an_unbound_binding_shows_its_path.md")]
#[test]
fn an_unbound_binding_shows_its_path() {
    let screen = OkuiScreen::compile(r#"<div class="counter" test-id="v">{cuont}</div>"#)
        .expect("the screen compiles");
    let (mut runtime, _data, _control, _css) = host(screen);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "v"), "cuont");
}

#[test]
fn runtime_virtual_rows_follow_data_keys_and_scroll() {
    let data = ScreenData::new();
    let mut rows: Vec<ScreenRow> = (0..10_000)
        .map(|index| {
            [
                ("id".into(), format!("row-{index}").into()),
                ("label".into(), format!("Item {index}").into()),
            ]
            .into_iter()
            .collect()
        })
        .collect();
    data.set_rows("rows", rows.clone());
    data.set("pending", true);
    let screen = OkuiScreen::compile_with(
        r#"<div class="scroller" test-id="list"><VirtualFor each={rows} as={row} key={row.id} loading={pending}>
            <div class="row" test-id={row.id}>{row.label}</div>
            <Loading><span test-id="pending">Fetching rows</span></Loading>
            <Empty><span test-id="empty">Nothing here</span></Empty>
            <Separator extent={4}><span test-id="divider"/></Separator>
        </VirtualFor></div>"#,
        data,
    )
    .expect("row names resolve in their loop scope");
    let (mut runtime, data, control, mut css) = host(screen);
    control.set_stylesheet(
        ".scroller { width: 200px; height: 120px; overflow: scroll }
         .row { height: 24px }",
    );
    control.apply(&mut runtime, &mut css);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "pending"), "Fetching rows");
    assert!(
        SelectorResolver::new()
            .resolve(&runtime, &SelectorTarget::test_id("row-0"))
            .is_none()
    );
    data.set("pending", false);
    frame(&mut runtime);
    let first = node(&runtime, "row-0");
    assert_eq!(rendered_text(&runtime, "row-0"), "Item 0");
    assert!(runtime.tree().iter().count() < 100);

    rows.swap(0, 1);
    rows[1].insert("label".into(), "Updated".into());
    data.set_rows("rows", rows);
    frame(&mut runtime);
    assert_eq!(node(&runtime, "row-0"), first);
    assert_eq!(rendered_text(&runtime, "row-0"), "Updated");

    let list = node(&runtime, "list");
    runtime.scroll_by(list, 2_800.0);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "row-100"), "Item 100");
    let mounted_dividers = runtime
        .tree()
        .iter()
        .filter(|(_, node)| {
            node.props().iter().any(|prop| {
                prop.name() == TEST_ID
                    && matches!(prop.value(), ViewValue::Text(id) if id.as_ref() == "divider")
            })
        })
        .count();
    assert!(
        mounted_dividers > 1,
        "visible rows mount repeated separators"
    );

    data.set_rows("rows", Vec::<ScreenRow>::new());
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "empty"), "Nothing here");
}

#[test]
fn runtime_conditionals_select_one_branch_and_react_to_changes() {
    let data = ScreenData::new();
    data.set("first", true);
    data.set("second", true);
    let screen = OkuiScreen::compile_with(
        r#"<div>
            <If cond={first}><span test-id="branch">First</span></If>
            <ElseIf cond={second}><span test-id="branch">Second</span></ElseIf>
            <Else><span test-id="branch">Fallback</span></Else>
            <If cond={false}><span test-id="never">Never</span></If>
            <If cond={true}><span test-id="literal">Literal</span></If>
        </div>"#,
        data,
    )
    .expect("conditional screen compiles");
    let (mut runtime, data, _, _) = host(screen);
    for (first, second, expected) in [
        (true, true, "First"),
        (false, true, "Second"),
        (false, false, "Fallback"),
    ] {
        data.set("first", first);
        data.set("second", second);
        frame(&mut runtime);
        assert_eq!(rendered_text(&runtime, "branch"), expected);
        assert_eq!(rendered_text(&runtime, "literal"), "Literal");
        assert!(
            SelectorResolver::new()
                .resolve(&runtime, &SelectorTarget::test_id("never"))
                .is_none()
        );
        assert_eq!(
            runtime
                .tree()
                .iter()
                .filter_map(|(_, node)| node.kind().text())
                .count(),
            2
        );
    }
}

#[test]
fn runtime_boolean_attribute_is_inactive_when_its_binding_is_removed() {
    let data = ScreenData::new();
    data.set("blocked", false);
    let screen = OkuiScreen::compile_with(
        r#"<button test-id="button" disabled={blocked}>Apply</button>"#,
        data,
    )
    .expect("conditional attribute compiles");
    let (mut runtime, data, _, _) = host(screen);
    let disabled = |runtime: &UiRuntime| {
        runtime
            .tree()
            .get(node(runtime, "button"))
            .unwrap()
            .attributes()
            .is_disabled()
    };
    frame(&mut runtime);
    assert!(!disabled(&runtime));
    data.set("blocked", true);
    frame(&mut runtime);
    assert!(disabled(&runtime));
    assert!(data.remove("blocked"));
    frame(&mut runtime);
    assert!(!disabled(&runtime));
}

#[test]
fn changing_a_runtime_node_key_replaces_its_retained_identity() {
    let data = ScreenData::new();
    data.set("session", "initial");
    let screen = OkuiScreen::compile_with(
        r#"<div><span test-id="keyed" key={session}>Session</span></div>"#,
        data,
    )
    .expect("key binding compiles");
    let (mut runtime, data, _, _) = host(screen);
    frame(&mut runtime);
    let initial = node(&runtime, "keyed");
    frame(&mut runtime);
    assert_eq!(node(&runtime, "keyed"), initial);

    data.set("session", "replacement");
    frame(&mut runtime);
    assert_ne!(node(&runtime, "keyed"), initial);
    assert!(runtime.tree().get(initial).is_none());
}

#[test]
fn runtime_for_uses_keys_and_collection_slots() {
    let row = |id: &str| ScreenRow::from([("id".into(), id.into())]);
    let data = ScreenData::new();
    data.set("pending", true);
    data.set_rows("rows", vec![row("a"), row("b")]);
    let screen = OkuiScreen::compile_with(
        r#"<div test-id="list"><For each={rows} as={record} key={record.id} loading={pending}>
            <span test-id={record.id}>{record.id}</span>
            <Separator><span test-id="separator">{index}</span></Separator>
            <Loading><span test-id="loading">Loading</span></Loading>
            <Empty><span test-id="empty">Empty</span></Empty>
        </For></div>"#,
        data,
    )
    .expect("the screen compiles");
    let (mut runtime, data, _, _) = host(screen);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "loading"), "Loading");

    data.set("pending", false);
    frame(&mut runtime);
    let a = node(&runtime, "a");
    let b = node(&runtime, "b");
    let separator = node(&runtime, "separator");
    let list = node(&runtime, "list");
    assert_eq!(
        runtime.tree().get(list).unwrap().child_nodes(),
        &[a, separator, b]
    );
    assert_eq!(rendered_text(&runtime, "separator"), "1");

    data.set_rows("rows", vec![row("b"), row("a")]);
    frame(&mut runtime);
    assert_eq!(node(&runtime, "a"), a);
    assert_eq!(node(&runtime, "b"), b);
    assert_eq!(runtime.tree().get(list).unwrap().child_nodes()[0], b);

    data.set_rows("rows", Vec::<ScreenRow>::new());
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "empty"), "Empty");
    assert_eq!(runtime.tree().get(list).unwrap().child_nodes().len(), 1);
}

#[doc = include_str!("okui_screen_state.doc/the_per_frame_lowering_is_far_cheaper_than_a_parse.md")]
#[test]
fn the_per_frame_lowering_is_far_cheaper_than_a_parse() {
    const ROUNDS: u32 = 200;
    let screen = OkuiScreen::compile(COUNTER).expect("the screen compiles");
    screen.data().set("count", 1_i64);

    // Warm both paths once so neither pays a first-call cost the other does not.
    let _ = screen.view();
    let _ = OkuiScreen::compile(COUNTER).expect("compiles");

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let view = screen.view();
        assert_eq!(view.root().len(), 1);
    }
    let lowering = started.elapsed() / ROUNDS;

    let started = Instant::now();
    for _ in 0..ROUNDS {
        let fresh = OkuiScreen::compile(COUNTER).expect("compiles");
        assert_eq!(fresh.document().node_count(), 4);
    }
    let compiling = started.elapsed() / ROUNDS;

    eprintln!(
        "[okui-screen] lowering {:?}/frame ({} nodes, {} of them data-dependent) vs \
         parse+compile {:?}",
        lowering,
        screen.document().node_count(),
        screen.dynamic_node_count(),
        compiling,
    );

    // The same measurement on a screen the size of a real one, so the number reported
    // is not about a four-node toy: 40 rows, every other one carrying a binding.
    let mut source = String::from("<div class=\"counter\">");
    for row in 0..40 {
        if row % 2 == 0 {
            source.push_str("<div class=\"button\">{count}</div>");
        } else {
            source.push_str("<div class=\"button\">static</div>");
        }
    }
    source.push_str("</div>");
    let big = OkuiScreen::compile(&source).expect("compiles");
    big.data().set("count", 1_i64);
    let _ = big.view();
    let started = Instant::now();
    for _ in 0..ROUNDS {
        let view = big.view();
        assert_eq!(view.root().len(), 1);
    }
    let big_lowering = started.elapsed() / ROUNDS;

    // And what the bridge alone costs for the same document — the floor any per-frame
    // lowering pays. The difference is the price of the data-aware path: a dynamic
    // subtree is lowered by the bridge and then has its children re-lowered, so the
    // overhead is bounded by the depth of the bound part of the tree, not by the
    // document.
    let started = Instant::now();
    for _ in 0..ROUNDS {
        let view = UiIrViewBridge::document_to_view(big.document());
        assert_eq!(view.root().len(), 1);
    }
    let bridge = started.elapsed() / ROUNDS;

    eprintln!(
        "[okui-screen] lowering {big_lowering:?}/frame ({} nodes, {} of them \
         data-dependent) vs the bridge alone {bridge:?}",
        big.document().node_count(),
        big.dynamic_node_count(),
    );
    assert!(
        big_lowering < bridge * 4,
        "the data-aware path must stay a small factor over the bridge it delegates to: \
         {big_lowering:?} vs {bridge:?}"
    );
    assert!(
        lowering * 4 < compiling,
        "keeping the IR must be much cheaper than re-parsing: lowering {lowering:?}, \
         parse+compile {compiling:?}"
    );
}

#[doc = include_str!("okui_screen_state.doc/a_static_screen_pays_nothing_for_the_data_aware_path.md")]
#[test]
fn a_static_screen_pays_nothing_for_the_data_aware_path() {
    let static_screen =
        OkuiScreen::compile(r#"<div class="counter"><span>hello</span></div>"#).expect("compiles");
    assert_eq!(static_screen.dynamic_node_count(), 0);

    let counter = OkuiScreen::compile(COUNTER).expect("compiles");
    assert_eq!(counter.document().node_count(), 4);
    assert_eq!(
        counter.dynamic_node_count(),
        3,
        "the root, the span and the interpolation — not the button beside them"
    );
}

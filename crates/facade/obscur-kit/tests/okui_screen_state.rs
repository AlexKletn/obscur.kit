//! A `.okui` screen that reflects changed state — the case that could not exist.
//!
//! §2.9 defines `.okui` as a front end "compiled into the same UI IR as the Rust
//! DSL", a peer of it. The run-time route was not a peer: `App::screen` compiled the
//! source in `run` and handed the runtime `move || view.clone()`, one pre-built
//! `View` for the application's whole life ("static v1 materialization"), so **no**
//! state change could reach the screen. A standalone `.okui` file and reactivity were
//! mutually exclusive.
//!
//! What changed is what is kept: the compiled **IR**, not the lowered `View`. Each
//! frame lowers it again — nothing re-parses, nothing re-compiles — and the lowering
//! resolves `{bindings}` against a [`ScreenData`] table the application writes. The
//! last case measures what that costs.
//!
//! Structure is still fixed: `<If>`/`<For>` are statically materialised on this
//! route, and a `{binding}` inside a `<VirtualFor>` keeps its placeholder. Shape that
//! depends on state belongs in the build-time route (a `build.rs` emitting through
//! `CodegenRoot::facade`, or the inline `okui!` macro), where the markup becomes
//! ordinary Rust — and where §2.9's compile-time contract validation happens.
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
use obscur_kit::screen::{OkuiScreen, ScreenData};

/// The handler name the markup binds its click to, hashed the same way the front end
/// hashes it.
const INCREMENT: InternedSymbol = InternedSymbol::named("increment");

/// A counter, in markup: the value is a body binding, and the button is an ordinary
/// element with a click binding.
const COUNTER: &str = r#"<div class="counter">
    <span class="value" test-id="value">{count}</span>
    <div class="button" test-id="plus" on:click=increment/>
</div>"#;

const CSS: &str = ".counter { width: 200px; height: 100px } \
                   .value { width: 200px; height: 50px } \
                   .button { width: 200px; height: 50px }";

/// A runtime whose root is `screen`, with the CSS installed and one frame committed.
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

/// Runs one reactive frame — the builder is invoked, so the screen is lowered again.
fn frame(runtime: &mut UiRuntime) {
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("a requested frame is due");
}

/// The node carrying `test-id`, resolved the way §7.5 says to.
fn node(runtime: &UiRuntime, test_id: &str) -> NodeId {
    SelectorResolver::new()
        .resolve(runtime, &SelectorTarget::test_id(test_id))
        .unwrap_or_else(|| panic!("no node carries test-id=\"{test_id}\""))
}

/// The text content **rendered under** the element carrying `test-id`, read from the
/// retained tree rather than from the desired view — so what is asserted is what the
/// runtime committed.
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

/// A trusted click at `(x, y)`.
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

/// **The case that could not exist.** One compiled screen, two frames, two values.
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

/// The whole loop, from markup to pixels: the screen's own `on:click=increment`
/// dispatches the command, the handler writes the table, and the next frame's
/// lowering shows it. A counter in a standalone `.okui` file, which is exactly what
/// was impossible.
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

/// A bound **attribute** reaches the cascade, not just the tree: `class={theme}` is
/// substituted before reconcile, so the matcher sees the class the data holds and the
/// element is styled by it.
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

/// An unbound binding keeps its path visible rather than blanking, so a misspelling is
/// a wrong word on screen instead of a missing line.
#[test]
fn an_unbound_binding_shows_its_path() {
    let screen = OkuiScreen::compile(r#"<div class="counter" test-id="v">{cuont}</div>"#)
        .expect("the screen compiles");
    let (mut runtime, _data, _control, _css) = host(screen);
    frame(&mut runtime);
    assert_eq!(rendered_text(&runtime, "v"), "cuont");
}

/// What the per-frame lowering costs, and the claim it backs: keeping the **IR** and
/// lowering it again is cheap, where re-parsing the source per frame — the obvious
/// wrong answer — is not.
///
/// The numbers are printed (`cargo test -- --nocapture`) because a wall-clock
/// threshold is not a property worth asserting on someone else's machine. What *is*
/// asserted is the ratio the design rests on: a lowering costs a fraction of a
/// parse+compile, so a screen can be rebuilt every frame and a source cannot.
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

/// The cost model in one assertion: a screen with nothing bound is walked by the
/// bridge alone, and only the path from the root to a binding is re-lowered.
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

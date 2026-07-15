//! **Headless proof: the interactive Wireframe-Kit demo WORKS — and the bare
//! controls get their look entirely from the installed UA default stylesheet.**
//!
//! The deterministic, display-free sibling of `wireframe_live_window.rs`. It boots
//! the shared [`wireframe`] scene on a real [`UiRuntime`] with the kit's **UA
//! default stylesheet installed** (`runtime.set_default_stylesheet(
//! obscur_kit::ua_stylesheet())`) — the ONLY thing that styles the bare controls —
//! plus the demo's LAYOUT-CHROME author sheet on top, then drives every interaction
//! **programmatically through the real input path** ([`handle_pointer`] for hover /
//! press / release / click, [`handle_key`] for typing) and rasterizes two stitched
//! panels to `target/wireframe_live_proof.png`:
//!
//! - **Panel 1 — Idle.** The spec sheet at rest: masthead, seven section cards, and
//!   the footer, every bare control showing its neutral UA base look.
//! - **Panel 2 — Interacted.** The button hovered + pressed (`:hover` + `:active`
//!   + `:focus` all biting from the live pointer/press/focus targets), the field
//!   focused with typed text, the checkbox toggled `:checked` (box + `✓`), the
//!   tri-state advanced, one radio selected (filled dot), and the select opened +
//!   picked (arrow flip + chosen label).
//!
//! Each interaction is located by hit-testing the runtime's own layout boxes (no
//! pixel positions are hard-coded), driven through the pointer/keyboard API, and
//! **asserted**, so a regression fails the run rather than producing a wrong image.
//! The disabled button is clicked and shown to be a genuine no-op.
//!
//! Run it:
//! `cargo run -p obscur-kit --example wireframe_live_headless \
//!   --features "render-cpu text-harfrust text-unicode"`
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, PhysicalKey};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;

#[path = "wireframe_live/scene.rs"]
mod wireframe;

use wireframe::{
    CMD_AREA, CMD_BTN, CMD_CHK, CMD_FIELD, CMD_LINK, CMD_SEL, CMD_TRI, RADIO_COUNT, RADIO_LABELS,
    SELECT_COUNT, WireState, radio_command,
};

/// The logical extent each panel is laid out and rasterized under. Tall enough for
/// the whole spec sheet (masthead + 7 cards + footer) plus the select's open list.
const VIEWPORT: Size = Size::new(1240.0, 1560.0);

/// A long single-line value typed into the text field — wider than the field's
/// definite 220px, so it proves an `<input>` is single-line (clips, never wraps).
const FIELD_TEXT: &str = "The quick brown fox jumped over";
/// A long value typed into the textarea — long enough to wrap onto several lines
/// within the definite 360px box (proving the wrap width is the box, not min-content).
const AREA_TEXT: &str = "The quick brown fox jumps over the lazy dog and then it runs far away again today while the sun sets slowly behind the distant hills";

fn main() {
    println!("== obscur-kit interactive Wireframe Kit — headless proof (CPU) ==\n");

    // The UA default sheet — the ONLY thing that styles the bare controls.
    let ua = obscur_kit::ua_stylesheet();
    println!(
        "[ua] installed default sheet: {} rule(s) at {:?}",
        ua.rules().len(),
        ua.origin()
    );

    let state = Rc::new(RefCell::new(WireState::default()));
    // Author sheet = LAYOUT CHROME ONLY (masthead / cards / captions / page). It
    // carries NO control-appearance rule, so the UA sheet shows through.
    let sheets = vec![compile(wireframe::stylesheet())];

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // Install the UA default at the lowest cascade origin — bare controls get the
    // wireframe look from THIS, matched below the layout-chrome author sheet.
    runtime.set_default_stylesheet(ua);

    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    } else {
        eprintln!("no system font found; glyph indicators (✓ ▼ ▲) will not render");
    }

    register_handlers(&mut runtime, &state);

    // --- Panel 1: idle -----------------------------------------------------
    refresh(&mut runtime, &state, &sheets);
    let panel_idle = capture(&mut runtime, &mut renderer);
    println!("[panel 1] idle captured");

    // --- Toggles: checkbox, tri-state, radio -------------------------------
    click(&mut runtime, &state, &sheets, "chk");
    assert!(
        state.borrow().check.is_checked(),
        "checkbox toggled to checked"
    );

    click(&mut runtime, &state, &sheets, "chk-tri");
    assert!(
        state.borrow().tri.is_checked(),
        "tri-state advanced from mixed to checked"
    );

    click(&mut runtime, &state, &sheets, "radio-1");
    assert_eq!(
        state.borrow().radio,
        1,
        "radio group selected option 1 (Medium)"
    );

    // --- Select: open → pick-next → reopen so the popup shows a selected row ---
    click(&mut runtime, &state, &sheets, "select");
    assert!(state.borrow().select.is_open(), "select opened (arrow ▲)");

    click(&mut runtime, &state, &sheets, "select");
    assert!(
        !state.borrow().select.is_open(),
        "select closed after picking"
    );
    assert_eq!(
        state.borrow().select.selected_index(),
        Some(0),
        "select committed the first option on pick-next from an empty selection"
    );

    // Reopen so the interacted panel captures the dropdown popup WITH a selected
    // (inverted) row, and assert the popup renders as a vertical stack of option
    // rows positioned directly BELOW the header.
    click(&mut runtime, &state, &sheets, "select");
    assert!(
        state.borrow().select.is_open(),
        "select reopened for the popup capture"
    );
    assert_select_popup(&runtime);

    // --- Field: type a LONG string; a single-line `input` must NOT wrap --------
    let field = node_by_id(&runtime, "field").expect("field node");
    let field_pt = center(&runtime, "field").expect("field laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Down, field_pt));
    runtime.handle_pointer(pointer(PointerEventKind::Up, field_pt));
    let seed = state.borrow().field.clone();
    runtime.begin_text_editing(field, seed);
    type_text(&mut runtime, FIELD_TEXT);
    refresh(&mut runtime, &state, &sheets);
    assert!(
        state.borrow().field.ends_with(FIELD_TEXT),
        "typed characters reached the field (got {:?})",
        state.borrow().field
    );
    let (_, field_size) = box_of(&runtime, "field").expect("field laid out");
    println!(
        "[measure] field box = {:.1} × {:.1}",
        field_size.width, field_size.height
    );
    assert!(
        (field_size.width - 220.0).abs() < 2.0,
        "field keeps its definite 220px width (got {:.1})",
        field_size.width,
    );
    assert!(
        field_size.height < 56.0,
        "single-line <input> did NOT wrap the long value — box stays one line high \
         (got {:.1}; a wrapped 2-line box would be ~66+)",
        field_size.height,
    );

    // --- Textarea: type a LONG string; `.wf-area` (white-space:normal, 360×96)
    // must WRAP within its definite box, not early at min-content --------------
    let area = node_by_id(&runtime, "area").expect("area node");
    let area_pt = center(&runtime, "area").expect("area laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Down, area_pt));
    runtime.handle_pointer(pointer(PointerEventKind::Up, area_pt));
    let area_seed = state.borrow().area.clone();
    runtime.begin_text_editing(area, area_seed);
    type_text(&mut runtime, AREA_TEXT);
    refresh(&mut runtime, &state, &sheets);
    assert!(
        state.borrow().area.ends_with(AREA_TEXT),
        "typed characters reached the textarea (got {:?})",
        state.borrow().area
    );
    let (_, area_size) = box_of(&runtime, "area").expect("area laid out");
    let (_, area_text) = text_child_box(&runtime, "area").expect("area text laid out");
    println!(
        "[measure] area box = {:.1} × {:.1}; wrapped text = {:.1} × {:.1} (inner width ~334)",
        area_size.width, area_size.height, area_text.width, area_text.height,
    );
    assert!(
        (area_size.width - 360.0).abs() < 2.0,
        "textarea keeps its definite 360px width (got {:.1})",
        area_size.width,
    );
    assert!(
        area_text.width > 280.0,
        "textarea wraps at its REAL ~360px box (inner ~334), NOT early at min-content \
         — widest laid-out line = {:.1}. If this is small (~a single word wide) with a \
         tall text box, that is the definite-width→text-wrap propagation bug (DEFECT 1 \
         step 3): report it, do not hack around it",
        area_text.width,
    );
    assert!(
        area_text.height > 24.0,
        "textarea wrapped onto MULTIPLE lines within its box — text height {:.1} exceeds a \
         single ~17px line (the field's single-line text)",
        area_text.height,
    );

    // --- Hover the link (drives the `a:hover` bold) ------------------------
    let link_pt = center(&runtime, "link").expect("link laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Move, link_pt));

    // --- Button: hover + press (hold) so :hover + :active + :focus show -----
    // Rendered WITHOUT releasing, so all three pseudo-states bite at once. The
    // recomputed states come straight from the runtime's live targets.
    let btn = center(&runtime, "btn").expect("button laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Move, btn));
    runtime.handle_pointer(pointer(PointerEventKind::Down, btn));
    refresh(&mut runtime, &state, &sheets);
    let panel_interacted = capture(&mut runtime, &mut renderer);
    println!("[panel 2] hover·active·focus button + checked + selected + open→picked captured");

    // Release completes the click → the button's command bumps the counter.
    runtime.handle_pointer(pointer(PointerEventKind::Up, btn));
    refresh(&mut runtime, &state, &sheets);
    assert_eq!(state.borrow().clicks, 1, "button release fired one click");

    // --- Disabled control is a genuine no-op -------------------------------
    let before = state.borrow().clicks;
    click(&mut runtime, &state, &sheets, "btn-off");
    assert_eq!(
        state.borrow().clicks,
        before,
        "clicking the disabled button changed nothing (its command has no handler)"
    );
    println!("[check] disabled button click is a no-op");

    // --- Stitch the two panels side by side and write the PNG --------------
    let combined = stitch(&[&panel_idle, &panel_interacted]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("wireframe_live_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — idle | interacted (bare controls, UA look)",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
    // --- Demo scroll container: the sheet scrolls in a REAL overflow container --
    verify_demo_scrolls();

    println!("== all interaction assertions held; exiting 0 ==");
}

/// The window-sized viewport the scroll check lays the sheet out under: shorter
/// than the ~1560-px sheet, so `#app` (`height: 100%; overflow: auto`) becomes a
/// real scroll container the wheel drives — the same rewire the windowed demo uses
/// (CSS Overflow Module Level 3 §scroll-container).
const SCROLL_VIEWPORT: Size = Size::new(1080.0, 720.0);

/// Proves the demo's `#app` page is a genuine vertical scroll container: it clips +
/// scrolls its content (the `.wf-col` sheet) via `scroll_by`, exactly as the
/// windowed driver's mouse wheel does, and paints a scrollbar. Rasterizes an
/// idle|scrolled pair of the window-sized sheet to `target/wireframe_scroll_proof.png`.
fn verify_demo_scrolls() {
    let sheets = vec![compile(wireframe::stylesheet())];
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(3), SCROLL_VIEWPORT);
    runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());
    let mut renderer = CpuRenderer::new(RendererId::from_raw(4));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    }
    let state = Rc::new(RefCell::new(WireState::default()));

    let view = wireframe::view(&state.borrow());
    runtime.render_now(&view, &sheets);

    // `#app` is the root's single child (the page). With a 720-px viewport and a
    // ~1560-px sheet it overflows → a scroll container starting at scroll_top 0.
    let root = runtime.tree().root().expect("root after a frame");
    let app = runtime.tree().get(root).unwrap().children()[0];
    assert_eq!(runtime.scroll_top(app), 0.0, "the page starts unscrolled");
    let idle = capture_at(&mut runtime, &mut renderer, SCROLL_VIEWPORT);

    // Drive it exactly as the wheel does: scroll the sheet down by 400 logical px.
    let landed = runtime.scroll_by(app, 400.0);
    assert!(
        landed > 0.0,
        "#app scrolls — its content overflows the viewport (got {landed})"
    );
    assert_eq!(runtime.scroll_top(app), landed, "scroll_top advanced");
    runtime.render_now(&view, &sheets);

    // The scrolled frame carries a translate(0,-scroll_top) wrapping the content.
    let snapshot = runtime.snapshot();
    let scrolled_by = snapshot
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            obscur_kit::scene::command::PaintCommand::PushTransform(t) if t.coeffs[5] < 0.0 => {
                Some(-t.coeffs[5])
            }
            _ => None,
        });
    assert_eq!(
        scrolled_by,
        Some(landed),
        "the sheet is translated up by scroll_top"
    );
    let scrolled = capture_at(&mut runtime, &mut renderer, SCROLL_VIEWPORT);

    let combined = stitch(&[&idle, &scrolled]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("wireframe_scroll_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "[scroll] #app is a real overflow container — scrolled {landed:.0}px; wrote {} → {}",
        format_args!("{} × {}", combined.width(), combined.height()),
        out.display(),
    );
}

/// Rasterizes the runtime's committed scene into a fresh pixmap sized to `size`.
fn capture_at(
    runtime: &mut UiRuntime,
    renderer: &mut CpuRenderer,
    size: Size,
) -> tiny_skia::Pixmap {
    let snapshot = runtime.snapshot();
    let frame = RenderFrame::new(snapshot, size);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(size));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

// ---------------------------------------------------------------------------
// Command handlers — identical semantics to the windowed driver.
// ---------------------------------------------------------------------------

/// Registers a handler per interactive command, all mutating the shared state and
/// refreshing its status line (the disabled controls' commands are deliberately
/// left unregistered so they are no-ops).
fn register_handlers(runtime: &mut UiRuntime, state: &Rc<RefCell<WireState>>) {
    {
        let s = state.clone();
        runtime.on_command(CMD_BTN, move || {
            let mut st = s.borrow_mut();
            st.clicks += 1;
            st.last_event = format!("Button clicked (#{})", st.clicks);
        });
    }
    {
        let s = state.clone();
        runtime.on_command(CMD_CHK, move || {
            let mut st = s.borrow_mut();
            st.check.toggle();
            st.last_event = "Checkbox toggled".to_owned();
        });
    }
    {
        let s = state.clone();
        runtime.on_command(CMD_TRI, move || {
            let mut st = s.borrow_mut();
            st.tri.toggle();
            st.last_event = "Tri-state advanced".to_owned();
        });
    }
    for i in 0..RADIO_COUNT {
        let s = state.clone();
        runtime.on_command(radio_command(i), move || {
            let mut st = s.borrow_mut();
            st.radio = i;
            st.last_event = format!("Radio → {}", RADIO_LABELS[i]);
        });
    }
    {
        let s = state.clone();
        runtime.on_command(CMD_SEL, move || {
            let mut st = s.borrow_mut();
            if st.select.is_open() {
                // Bare `Select` bubbles one event for the whole control, so "pick"
                // is a cycle-to-next (per-option hit-routing is not a bare-widget
                // capability). select() commits + closes.
                let next = st
                    .select
                    .selected_index()
                    .map_or(0, |i| (i + 1) % SELECT_COUNT);
                st.select.select(next);
                st.last_event = format!("Selected {}", wireframe::SELECT_LABELS[next]);
            } else {
                st.select.open();
                st.last_event = "Select opened".to_owned();
            }
        });
    }
    {
        let s = state.clone();
        runtime.on_text_input(CMD_FIELD, move |text| {
            let mut st = s.borrow_mut();
            text.clone_into(&mut st.field);
            st.last_event = format!("Typed: {text:?}");
        });
    }
    {
        let s = state.clone();
        runtime.on_text_input(CMD_AREA, move |text| {
            let mut st = s.borrow_mut();
            text.clone_into(&mut st.area);
            st.last_event = format!("Typed area: {text:?}");
        });
    }
    {
        let s = state.clone();
        runtime.on_command(CMD_LINK, move || {
            s.borrow_mut().last_event = "Link activated".to_owned();
        });
    }
}

// ---------------------------------------------------------------------------
// Interaction + rendering helpers.
// ---------------------------------------------------------------------------

/// Rebuilds the view from the current state and runs one real frame through the
/// runtime (reconcile → match → layout → compose), refreshing the retained tree,
/// layout boxes, and spatial index so the next hit-test sees the current state.
fn refresh(runtime: &mut UiRuntime, state: &Rc<RefCell<WireState>>, sheets: &[Stylesheet]) {
    let view = wireframe::view(&state.borrow());
    runtime.render_now(&view, sheets);
}

/// Clicks the control identified by `id`: a full press + release just inside its
/// left edge (so the control element itself is the hit target, not its text child),
/// then a refresh so the mutated state is laid out.
fn click(runtime: &mut UiRuntime, state: &Rc<RefCell<WireState>>, sheets: &[Stylesheet], id: &str) {
    let p = center(runtime, id).unwrap_or_else(|| panic!("control {id:?} is laid out"));
    runtime.handle_pointer(pointer(PointerEventKind::Down, p));
    runtime.handle_pointer(pointer(PointerEventKind::Up, p));
    refresh(runtime, state, sheets);
}

/// Builds a mouse [`PointerEvent`] of `kind` at content-space point `p`.
fn pointer(kind: PointerEventKind, p: Point) -> PointerEvent {
    PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, p)
}

/// Types `text` into the focused editable, one printable character at a time
/// through the real keyboard path (press + release), so the runtime's editing
/// session inserts each character and fires the field's `on-input` handler.
fn type_text(runtime: &mut UiRuntime, text: &str) {
    for ch in text.chars() {
        let physical = physical_for(ch);
        let logical = Some(LogicalKey::character(ch));
        runtime.handle_key(KeyEvent::press(physical, logical));
        runtime.handle_key(KeyEvent::release(physical, logical));
    }
}

/// A plausible physical key position for a typed character (only the scan position;
/// the inserted character comes from the logical key). Falls back to `KeyA`.
fn physical_for(ch: char) -> PhysicalKey {
    match ch.to_ascii_lowercase() {
        'b' => PhysicalKey::KeyB,
        'c' => PhysicalKey::KeyC,
        'd' => PhysicalKey::KeyD,
        'e' => PhysicalKey::KeyE,
        _ => PhysicalKey::KeyA,
    }
}

/// Finds the [`NodeId`] of the element carrying the reserved `id` attribute `id`.
fn node_by_id(runtime: &UiRuntime, id: &str) -> Option<NodeId> {
    runtime
        .tree()
        .iter()
        .find(|(_, node)| node.attributes().id() == Some(id))
        .map(|(node_id, _)| node_id)
}

/// The click point for the control `id`: a point just inside its left edge (in the
/// padding, left of the label text) at the vertical centre, so the control element
/// itself is the topmost hit rather than its text child.
fn center(runtime: &UiRuntime, id: &str) -> Option<Point> {
    let node = node_by_id(runtime, id)?;
    let layout = runtime.layout_boxes().get(&node)?;
    let origin = layout.origin();
    let size = layout.size();
    Some(Point::new(origin.x + 6.0, origin.y + size.height / 2.0))
}

/// The layout box (origin, size) of the control carrying reserved `id`.
fn box_of(runtime: &UiRuntime, id: &str) -> Option<(Point, Size)> {
    let node = node_by_id(runtime, id)?;
    let lb = runtime.layout_boxes().get(&node)?;
    Some((lb.origin(), lb.size()))
}

/// The layout box of the first direct **Text** child of the control `id` (an
/// input/textarea renders its value as a single Text child; when it wraps, that
/// box spans multiple lines — width = widest line, height = total).
fn text_child_box(runtime: &UiRuntime, id: &str) -> Option<(Point, Size)> {
    let parent = node_by_id(runtime, id)?;
    let node = runtime.tree().get(parent)?;
    for &child in node.children() {
        let c = runtime.tree().get(child)?;
        if c.kind().text().is_some() {
            let lb = runtime.layout_boxes().get(&child)?;
            return Some((lb.origin(), lb.size()));
        }
    }
    None
}

/// Every node carrying the CSS `class`, in tree order.
fn nodes_by_class(runtime: &UiRuntime, class: &str) -> Vec<NodeId> {
    runtime
        .tree()
        .iter()
        .filter(|(_, n)| n.attributes().classes().iter().any(|c| &**c == class))
        .map(|(id, _)| id)
        .collect()
}

/// Asserts the OPEN select's popup renders as a real dropdown: a `.select-list`
/// container positioned directly BELOW the header, holding a vertical stack of
/// `.select-option` rows (one per option), the selected row carrying `selected`.
fn assert_select_popup(runtime: &UiRuntime) {
    let header = *nodes_by_class(runtime, "select-header")
        .first()
        .expect("the select renders a `.select-header` row");
    let header_lb = runtime
        .layout_boxes()
        .get(&header)
        .expect("header laid out");
    let (header_origin, header_size) = (header_lb.origin(), header_lb.size());
    let list = *nodes_by_class(runtime, "select-list")
        .first()
        .expect("an open select emits a `.select-list` popup");
    let list_lb = runtime
        .layout_boxes()
        .get(&list)
        .expect("popup list laid out");
    let list_top = list_lb.origin().y;
    let header_bottom = header_origin.y + header_size.height;
    // `top: 100%` anchors the list at the header's PADDING-box bottom, i.e. just
    // inside the 1.5px bottom border — so the list top lands within a couple of px
    // of the border-box bottom (never up at the header top: this is a real popup,
    // not an inline row).
    assert!(
        list_top >= header_bottom - 4.0,
        "the popup list sits BELOW the header (list top {:.1} vs header bottom {:.1})",
        list_top,
        header_bottom,
    );
    assert!(
        list_top > header_origin.y + header_size.height * 0.5,
        "the popup is a real dropdown below the header, not options inline at the top \
         (list top {:.1}, header spans {:.1}..{:.1})",
        list_top,
        header_origin.y,
        header_bottom,
    );

    // The option rows: one per option, stacked vertically (each strictly below the
    // previous, sharing a left edge), and exactly one carries the `selected` class.
    let options = nodes_by_class(runtime, "select-option");
    assert_eq!(
        options.len(),
        SELECT_COUNT,
        "one `.select-option` row per option"
    );
    let mut prev_bottom = f32::MIN;
    let mut left_edge: Option<f32> = None;
    let mut selected_rows = 0usize;
    for opt in &options {
        let lb = runtime
            .layout_boxes()
            .get(opt)
            .expect("option row laid out");
        let o = lb.origin();
        assert!(
            lb.size().height > 30.0,
            "each option row is a full-height row, not a collapsed line (height {:.1})",
            lb.size().height,
        );
        assert!(
            o.y >= prev_bottom - 1.0,
            "option rows are stacked vertically (row top {:.1} not below prev bottom {:.1})",
            o.y,
            prev_bottom,
        );
        prev_bottom = o.y + lb.size().height;
        match left_edge {
            None => left_edge = Some(o.x),
            Some(x) => assert!((o.x - x).abs() < 1.0, "option rows share a left edge"),
        }
        let node = runtime.tree().get(*opt).expect("option node");
        if node
            .attributes()
            .classes()
            .iter()
            .any(|c| &**c == "selected")
        {
            selected_rows += 1;
        }
    }
    assert_eq!(
        selected_rows, 1,
        "exactly one option row is marked `selected` (inverted)"
    );
    println!(
        "[check] select popup: {} rows stacked below header, one inverted",
        options.len()
    );
}

/// Rasterizes the runtime's current committed scene into a fresh pixmap.
fn capture(runtime: &mut UiRuntime, renderer: &mut CpuRenderer) -> tiny_skia::Pixmap {
    let snapshot = runtime.snapshot();
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Horizontally stitches `panels` left-to-right with thin dividers between them.
fn stitch(panels: &[&tiny_skia::Pixmap]) -> tiny_skia::Pixmap {
    let gap = 3u32;
    let height = panels.iter().map(|p| p.height()).max().unwrap_or(0);
    let width: u32 =
        panels.iter().map(|p| p.width()).sum::<u32>() + gap * (panels.len().max(1) as u32 - 1);
    let mut out = tiny_skia::Pixmap::new(width, height).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0x2b, 0x2a, 0x27, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut x = 0i32;
    for panel in panels {
        out.draw_pixmap(
            x,
            0,
            panel.as_ref(),
            &paint,
            tiny_skia::Transform::identity(),
            None,
        );
        x += panel.width() as i32 + gap as i32;
    }
    out
}

/// Compiles CSS text into a runtime-matchable [`Stylesheet`] (author origin).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}

/// Wires the text stack and registers `bytes` under `sans-serif` on both the
/// resolver and the CPU renderer so glyph handles line up.
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut CpuRenderer, bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();
    let id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        bytes.clone(),
    );
    renderer.register_font(font_handle(id), bytes);
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

/// The scene font handle (`u32`) for a resolved [`FontId`].
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes, scanning well-known locations (best effort).
fn load_system_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}

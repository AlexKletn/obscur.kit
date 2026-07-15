//! **Headless proof: the interactive control gallery WORKS without a display.**
//!
//! The deterministic, display-free sibling of `controls_window.rs`. It boots the
//! very same shared [`controls`] scene on a real [`UiRuntime`], then drives every
//! interaction **programmatically through the real input path** —
//! [`handle_pointer`](UiRuntime::handle_pointer) for hover / press / release /
//! click and [`handle_key`](UiRuntime::handle_key) for typing — and rasterizes the
//! resulting states through the software [`CpuRenderer`] into three stitched
//! panels written to `target/controls_gallery_proof.png`:
//!
//! - **Panel 1 — Idle.** The gallery at rest: nothing hovered, focused, or
//!   toggled.
//! - **Panel 2 — Hover · Active · Open.** The checkboxes toggled `:checked`, the
//!   radio group with one option `:checked`, the **Select popup open** (its option
//!   list rendered from the real [`Select::is_open`] state), and the primary
//!   button held **pressed** so `:hover` + `:active` + `:focus` all bite from the
//!   runtime's live pointer/press/focus targets.
//! - **Panel 3 — Typed · Selected.** The button released (click count bumped), the
//!   Select collapsed onto its chosen option, and the text field **focused with a
//!   caret** showing text the run typed through the editing session.
//!
//! Each interaction is located by hit-testing the runtime's own layout boxes (so
//! no pixel positions are hard-coded), driven through the pointer/keyboard API,
//! and asserted, so a regression fails the run rather than producing a wrong
//! image. The disabled button is clicked and shown to be a genuine no-op.
//!
//! Run it:
//! `cargo run -p obscur-kit --example controls_headless \
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

#[path = "controls/scene.rs"]
mod controls;

use controls::{
    CMD_BTN, CMD_CHK, CMD_FIELD, CMD_SEL_TOGGLE, CMD_TRI, GalleryState, RADIO_COUNT, RADIO_LABELS,
    SELECT_COUNT, SELECT_LABELS, radio_command, select_option_command,
};

/// The logical extent each panel is laid out and rasterized under. Tall enough to
/// show the whole gallery **with the Select popup expanded** without clipping.
const VIEWPORT: Size = Size::new(660.0, 860.0);

fn main() {
    println!("== obscur-kit interactive control gallery — headless proof (CPU) ==\n");

    let state = Rc::new(RefCell::new(GalleryState::default()));
    let sheets = vec![compile(controls::stylesheet())];

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    if let Some(bytes) = load_system_font() {
        wire_text_stack(&mut runtime, &mut renderer, bytes);
    } else {
        eprintln!("no system font found; text will be blank (layout still exercised)");
    }

    register_handlers(&mut runtime, &state);

    // --- Panel 1: idle -----------------------------------------------------
    refresh(&mut runtime, &state, &sheets);
    let panel_idle = capture(&mut runtime, &mut renderer);
    println!("[panel 1] idle captured");

    // --- Toggles: checkbox, tri-state, radio, open the select --------------
    click(&mut runtime, &state, &sheets, "chk");
    assert!(state.borrow().check.is_checked(), "checkbox toggled to checked");

    click(&mut runtime, &state, &sheets, "chk-tri");
    assert!(
        state.borrow().tri.is_checked(),
        "tri-state advanced from indeterminate to checked"
    );

    click(&mut runtime, &state, &sheets, "radio-1");
    assert_eq!(state.borrow().radio, 1, "radio group selected option 1");

    click(&mut runtime, &state, &sheets, "sel-header");
    assert!(state.borrow().select.is_open(), "select popup opened");

    // --- Panel 2: hover + active + focus on the button, popup open ---------
    // Real pointer path: move over the button (hover), press (focus + active) and
    // hold — then render WITHOUT releasing so `:hover` + `:active` + `:focus` all
    // show. The recomputed pseudo-states come straight from the runtime.
    let btn = center(&runtime, "btn").expect("button laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Move, btn));
    runtime.handle_pointer(pointer(PointerEventKind::Down, btn));
    render_current(&mut runtime, &state, &sheets); // apply pseudo-state to the paint
    let panel_hover = capture(&mut runtime, &mut renderer);
    println!("[panel 2] hover + active + focus + open select captured");

    // Release completes the click → the button's command bumps the counter.
    runtime.handle_pointer(pointer(PointerEventKind::Up, btn));
    refresh(&mut runtime, &state, &sheets);
    assert_eq!(state.borrow().click_count, 1, "button release fired one click");

    // --- Pick a select option (closes the popup onto option 1) -------------
    click(&mut runtime, &state, &sheets, "sel-opt-1");
    assert_eq!(
        state.borrow().select.selected_index(),
        Some(1),
        "select committed option 1"
    );
    assert!(!state.borrow().select.is_open(), "select closed after picking");

    // --- Focus the field + type through the editing session ----------------
    let field = node_by_id(&runtime, "field").expect("field node");
    let field_pt = center(&runtime, "field").expect("field laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Down, field_pt));
    runtime.handle_pointer(pointer(PointerEventKind::Up, field_pt));
    // Seat the caret at the end of the existing text, then type — deterministic.
    let seed = state.borrow().input.clone();
    runtime.begin_text_editing(field, seed);
    type_text(&mut runtime, "abc");
    refresh(&mut runtime, &state, &sheets);
    assert!(
        state.borrow().input.ends_with("abc"),
        "typed characters reached the field via the editing session (got {:?})",
        state.borrow().input
    );
    assert_eq!(
        runtime.focused(),
        Some(field),
        "the field holds focus (drives the :focus ring + caret)"
    );

    let panel_typed = capture(&mut runtime, &mut renderer);
    println!("[panel 3] typed + selected + closed captured");

    // --- Disabled control is a genuine no-op -------------------------------
    let before = state.borrow().click_count;
    click(&mut runtime, &state, &sheets, "btn-off");
    assert_eq!(
        state.borrow().click_count,
        before,
        "clicking the disabled button changed nothing (its command has no handler)"
    );
    println!("[check] disabled button click is a no-op");

    // --- Stitch the three panels side by side and write the PNG ------------
    let combined = stitch(&[&panel_idle, &panel_hover, &panel_typed]);
    let png = combined.encode_png().expect("encode png");
    let out = output_path("controls_gallery_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — idle | hover·active·open | typed·selected",
        combined.width(),
        combined.height(),
        out.display(),
        png.len(),
    );
    println!("== all interaction assertions held; exiting 0 ==");
}

// ---------------------------------------------------------------------------
// Command handlers — identical semantics to the windowed driver.
// ---------------------------------------------------------------------------

/// Registers a handler per interactive command, all mutating the shared state and
/// refreshing its status line (the disabled controls' commands are deliberately
/// left unregistered so they are no-ops).
fn register_handlers(runtime: &mut UiRuntime, state: &Rc<RefCell<GalleryState>>) {
    {
        let s = state.clone();
        runtime.on_command(CMD_BTN, move || {
            let mut st = s.borrow_mut();
            st.click_count += 1;
            st.last_event = format!("Button clicked (#{})", st.click_count);
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
        runtime.on_command(CMD_SEL_TOGGLE, move || {
            let mut st = s.borrow_mut();
            st.select.toggle_open();
            st.last_event =
                format!("Select {}", if st.select.is_open() { "opened" } else { "closed" });
        });
    }
    for i in 0..SELECT_COUNT {
        let s = state.clone();
        runtime.on_command(select_option_command(i), move || {
            let mut st = s.borrow_mut();
            st.select.select(i);
            st.last_event = format!("Selected {}", SELECT_LABELS[i]);
        });
    }
    {
        let s = state.clone();
        runtime.on_text_input(CMD_FIELD, move |text| {
            let mut st = s.borrow_mut();
            text.clone_into(&mut st.input);
            st.last_event = format!("Typed: {text:?}");
        });
    }
}

// ---------------------------------------------------------------------------
// Interaction + rendering helpers.
// ---------------------------------------------------------------------------

/// Rebuilds the view from the current state and runs one real frame through the
/// runtime (reconcile → match → layout → compose), refreshing the retained tree,
/// layout boxes, and spatial index so the next hit-test sees the current state.
fn refresh(runtime: &mut UiRuntime, state: &Rc<RefCell<GalleryState>>, sheets: &[Stylesheet]) {
    let view = controls::view(&state.borrow());
    runtime.render_now(&view, sheets);
}

/// Rebuilds + renders like [`refresh`] but is named for the pseudo-state panels:
/// it re-applies the runtime's *current* interaction pseudo-states (hover / active
/// / focus, already recomputed by the preceding pointer events) to the paint.
fn render_current(runtime: &mut UiRuntime, state: &Rc<RefCell<GalleryState>>, sheets: &[Stylesheet]) {
    refresh(runtime, state, sheets);
}

/// Clicks the control identified by `id`: a full press + release on its box (in
/// the padding region, left of the label, so the control element itself is the hit
/// target — not its text child), then a refresh so the mutated state is laid out.
fn click(runtime: &mut UiRuntime, state: &Rc<RefCell<GalleryState>>, sheets: &[Stylesheet], id: &str) {
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

/// A plausible physical key position for a typed ASCII letter (only the scan
/// position; the inserted character comes from the logical key). Falls back to
/// `KeyA` for anything outside `a..=z`.
fn physical_for(ch: char) -> PhysicalKey {
    match ch.to_ascii_lowercase() {
        'a' => PhysicalKey::KeyA,
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

/// The click point for the control `id`: a point just inside its left edge
/// (in the padding, left of the label text) at the vertical centre, so the
/// control element itself is the topmost hit rather than its text child.
fn center(runtime: &UiRuntime, id: &str) -> Option<Point> {
    let node = node_by_id(runtime, id)?;
    let layout = runtime.layout_boxes().get(&node)?;
    let origin = layout.origin();
    let size = layout.size();
    Some(Point::new(origin.x + 6.0, origin.y + size.height / 2.0))
}

/// Rasterizes the runtime's current committed scene into a fresh pixmap.
fn capture(runtime: &mut UiRuntime, renderer: &mut CpuRenderer) -> tiny_skia::Pixmap {
    let snapshot = runtime.snapshot();
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    renderer.render(&submission, &target).expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Horizontally stitches `panels` left-to-right with thin dividers between them.
fn stitch(panels: &[&tiny_skia::Pixmap]) -> tiny_skia::Pixmap {
    let gap = 3u32;
    let height = panels.iter().map(|p| p.height()).max().unwrap_or(0);
    let width: u32 =
        panels.iter().map(|p| p.width()).sum::<u32>() + gap * (panels.len().max(1) as u32 - 1);
    let mut out = tiny_skia::Pixmap::new(width, height).expect("combined pixmap");
    out.fill(tiny_skia::Color::from_rgba8(0x2a, 0x2e, 0x38, 0xff));
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

/// Compiles CSS text into a runtime-matchable [`Stylesheet`].
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

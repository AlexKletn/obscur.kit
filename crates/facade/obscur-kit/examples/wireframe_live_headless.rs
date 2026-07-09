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
    CMD_BTN, CMD_CHK, CMD_FIELD, CMD_LINK, CMD_SEL, CMD_TRI, RADIO_COUNT, RADIO_LABELS,
    SELECT_COUNT, WireState, radio_command,
};

/// The logical extent each panel is laid out and rasterized under. Tall enough for
/// the whole spec sheet (masthead + 7 cards + footer) plus the select's open list.
const VIEWPORT: Size = Size::new(1240.0, 1560.0);

fn main() {
    println!("== obscur-kit interactive Wireframe Kit — headless proof (CPU) ==\n");

    // The UA default sheet — the ONLY thing that styles the bare controls.
    let ua = obscur_kit::ua_stylesheet();
    println!("[ua] installed default sheet: {} rule(s) at {:?}", ua.rules().len(), ua.origin());

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
    assert!(state.borrow().check.is_checked(), "checkbox toggled to checked");

    click(&mut runtime, &state, &sheets, "chk-tri");
    assert!(state.borrow().tri.is_checked(), "tri-state advanced from mixed to checked");

    click(&mut runtime, &state, &sheets, "radio-1");
    assert_eq!(state.borrow().radio, 1, "radio group selected option 1 (Medium)");

    // --- Select: open, then pick-next + close ------------------------------
    click(&mut runtime, &state, &sheets, "select");
    assert!(state.borrow().select.is_open(), "select opened (arrow ▲, options inline)");

    click(&mut runtime, &state, &sheets, "select");
    assert!(!state.borrow().select.is_open(), "select closed after picking");
    assert_eq!(
        state.borrow().select.selected_index(),
        Some(0),
        "select committed the first option on pick-next from an empty selection"
    );

    // --- Focus the field + type through the editing session ----------------
    let field = node_by_id(&runtime, "field").expect("field node");
    let field_pt = center(&runtime, "field").expect("field laid out");
    runtime.handle_pointer(pointer(PointerEventKind::Down, field_pt));
    runtime.handle_pointer(pointer(PointerEventKind::Up, field_pt));
    let seed = state.borrow().field.clone();
    runtime.begin_text_editing(field, seed);
    type_text(&mut runtime, "Typing");
    refresh(&mut runtime, &state, &sheets);
    assert!(
        state.borrow().field.ends_with("Typing"),
        "typed characters reached the field via the editing session (got {:?})",
        state.borrow().field
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
    println!("== all interaction assertions held; exiting 0 ==");
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
                let next = st.select.selected_index().map_or(0, |i| (i + 1) % SELECT_COUNT);
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
    out.fill(tiny_skia::Color::from_rgba8(0x2b, 0x2a, 0x27, 0xff));
    let paint = tiny_skia::PixmapPaint::default();
    let mut x = 0i32;
    for panel in panels {
        out.draw_pixmap(x, 0, panel.as_ref(), &paint, tiny_skia::Transform::identity(), None);
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
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(HarfrustShaper::without_font()));
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

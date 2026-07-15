//! **Windowed interactive Wireframe-Kit demo → Vello (GPU, live).**
//!
//! A live winit 0.30 + Vello/wgpu window laid out like the imported *Browser
//! Elements Wireframe Kit* — a graph-paper page, a masthead card, and seven
//! numbered section cards — driving **BARE** form controls that get their entire
//! look from the kit's **UA default stylesheet**. The driver installs that sheet
//! with `runtime.set_default_stylesheet(obscur_kit::ua_stylesheet())`, so hovering,
//! clicking, focusing, typing, toggling, and opening the dropdown all show the
//! wireframe base look with live CSS pseudo-classes recomputed by the runtime from
//! the genuine input path:
//!
//! - **Button** — clicking fires `EVENT_CLICK` → `CMD_BTN`, bumping a click count in
//!   the status line; `:hover` (fill shift) / `:active` (invert) / `:focus` (dashed
//!   ring) are recomputed live. A disabled button ignores input.
//! - **Text Field / Textarea** — clicking seats the caret + `:focus` ring; typing /
//!   IME flows winit → converters → the runtime editing session → live text.
//! - **Select** — the header opens the list (arrow ▲); a click while open picks the
//!   next option and closes (arrow ▼, chosen label). A disabled select ignores input.
//! - **Checkbox** — a plain box, a tri-state box, and a disabled box; a click runs
//!   [`Checkbox::toggle`], and the reserved `:checked` fills the box with the `✓`.
//! - **Radio** — a live group of three sharing one selection (filled dot), plus a
//!   disabled one.
//! - **Text Link** — a bare `<a>` showing the UA underline + `:hover` bold.
//!
//! The demo's OWN stylesheet is **layout chrome only** (masthead / cards / captions
//! / graph-paper page) — it never restyles a control, so the UA sheet is the whole
//! source of the control look. The status line ("last event: … · clicks: N") updates
//! every interaction. The window is **device-scale aware**: the render scale is the
//! display's HiDPI `scale_factor` — **not tied to the window size** — so resizing the
//! window reflows the layout instead of zooming the whole UI up and down. The mouse
//! wheel scrolls, and `Ctrl+=`/`Ctrl+-`/`Ctrl+0` zoom on top (the exact driver shape
//! reused from `controls_window.rs` / `showcase_window.rs`).
//!
//! Run it (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example wireframe_live_window --features full`
//!
//! # Environment note
//!
//! Like the other windowed demos this needs a display and a usable GPU adapter; in
//! a headless environment it compiles but exits cleanly on `resumed` when no
//! adapter/surface can be acquired (it never panics on that path). The headless CPU
//! proof (`wireframe_live_headless.rs`) is the display-free verification.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::devtools::{SharedInspectSource, live_source};
use obscur_kit::devtools_protocol::message::ScopeSet;
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::bridge::RuntimeBridge;
use obscur_kit::devtools_server::endpoint::DevToolsEndpoint;
use obscur_kit::devtools_server::transport::{SocketListener, SocketTransport};
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::RuntimeId;
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
use obscur_kit::perf::FrameProfiler;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text::ime::{CompositionEvent, Preedit};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::window_winit::conversion::{KeyboardConverter, PointerConverter};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "wireframe_live/scene.rs"]
mod wireframe;

use wireframe::{
    CMD_AREA, CMD_BTN, CMD_CHK, CMD_FIELD, CMD_LINK, CMD_SEL, CMD_TRI, RADIO_COUNT, RADIO_LABELS,
    SELECT_COUNT, WireState, radio_command,
};

/// One wheel *line* of scroll, in logical content px.
const WHEEL_LINE_STEP: f32 = 64.0;
/// The initial (and layout) window size in logical pixels.
const INITIAL_SIZE: Size = Size::new(1080.0, 760.0);

/// The winit application: owns the window, the GPU Vello renderer, the swapchain,
/// the embeddable [`UiRuntime`], and drives the shared [`WireState`].
struct WireframeWindow {
    window: Option<Arc<Window>>,
    surface: Option<VelloSurface>,
    device: Option<wgpu::Device>,
    renderer: Option<VelloRenderer>,
    runtime: UiRuntime,
    cursor: (f64, f64),
    scale_factor: f64,
    zoom: f32,
    modifiers: ModifiersState,
    profiler: FrameProfiler,
    inspect: SharedInspectSource,
}

impl WireframeWindow {
    /// Builds the app with a fresh [`UiRuntime`] wired to the shared scene: the
    /// **installed UA default sheet** (the only thing that styles the bare
    /// controls), the layout-chrome author sheet on top, the reactive view builder,
    /// and a handler per interactive command. All handlers mutate the shared
    /// [`WireState`] and refresh its status line so the events are visible.
    fn new() -> Self {
        let state = Rc::new(RefCell::new(WireState::default()));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);

        // THE POINT: install the kit's UA default stylesheet at the lowest cascade
        // origin. Every bare control (button / input / checkbox / radio / select /
        // <a>) gets the wireframe look from THIS sheet — the demo never restyles a
        // control. (`KitBootstrap::install_default_stylesheet` does the same thing.)
        runtime.set_default_stylesheet(obscur_kit::ua_stylesheet());

        // Button → bump the click counter.
        {
            let s = state.clone();
            runtime.on_command(CMD_BTN, move || {
                let mut st = s.borrow_mut();
                st.clicks += 1;
                st.last_event = format!("Button clicked (#{})", st.clicks);
            });
        }
        // Checkbox + tri-state → real toggle state machine.
        {
            let s = state.clone();
            runtime.on_command(CMD_CHK, move || {
                let mut st = s.borrow_mut();
                st.check.toggle();
                let on = st.check.is_checked();
                st.last_event = format!("Checkbox → {}", if on { "checked" } else { "unchecked" });
            });
        }
        {
            let s = state.clone();
            runtime.on_command(CMD_TRI, move || {
                let mut st = s.borrow_mut();
                st.tri.toggle();
                st.last_event = format!("Tri-state → {}", tri_word(&st.tri));
            });
        }
        // Radio group — one handler per option, sharing the selected index.
        for i in 0..RADIO_COUNT {
            let s = state.clone();
            runtime.on_command(radio_command(i), move || {
                let mut st = s.borrow_mut();
                st.radio = i;
                st.last_event = format!("Radio → {}", RADIO_LABELS[i]);
            });
        }
        // Select — one command for the whole bare widget: open when closed, else
        // pick-next + close (per-option hit-routing is not a bare-widget capability).
        {
            let s = state.clone();
            runtime.on_command(CMD_SEL, move || {
                let mut st = s.borrow_mut();
                if st.select.is_open() {
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
        // Input — per-keystroke text mirrored into state from the editing session.
        {
            let s = state.clone();
            runtime.on_text_input(CMD_FIELD, move |text| {
                let mut st = s.borrow_mut();
                text.clone_into(&mut st.field);
                st.last_event = format!("Typed: {text:?}");
            });
        }
        // Textarea — per-keystroke text mirrored into state, so the multi-line field
        // is a controlled field too (its value tracks the session every frame). With
        // the placeholder-override engine fix the uncontrolled case shows typed text
        // as well, but mirroring keeps the textarea's value in sync with the buffer.
        {
            let s = state.clone();
            runtime.on_text_input(CMD_AREA, move |text| {
                let mut st = s.borrow_mut();
                text.clone_into(&mut st.area);
                st.last_event = format!("Typed area: {text:?}");
            });
        }
        // Text link — activation updates the status line.
        {
            let s = state.clone();
            runtime.on_command(CMD_LINK, move || {
                s.borrow_mut().last_event = "Link activated".to_owned();
            });
        }
        // The reactive view builder + the layout-chrome author sheet (matched ABOVE
        // the installed UA default).
        {
            let s = state.clone();
            runtime.set_root_view(move || wireframe::view(&s.borrow()));
        }
        runtime.set_stylesheets(vec![compile(wireframe::stylesheet())]);

        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            zoom: 1.0,
            modifiers: ModifiersState::empty(),
            profiler: FrameProfiler::new(),
            inspect: SharedInspectSource::new(),
        }
    }

    /// The base device-pixel scale the UI renders at: the display's HiDPI
    /// [`scale_factor`](Self::scale_factor) — **independent of the window size**.
    ///
    /// The scale is *not* tied to the window width (no fit-to-width zoom): the UI
    /// renders at its natural CSS size for the display density, so widening or
    /// narrowing the window changes the logical viewport and the layout reflows,
    /// exactly like a real application — rather than scaling the whole UI up and
    /// down. The user `zoom` composes on top for an explicit magnify.
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    fn effective_scale(&self) -> f32 {
        self.base_scale() * self.zoom
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor.
    #[allow(clippy::cast_possible_truncation)]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
    }

    /// Re-syncs the runtime's layout viewport and device scale to the current
    /// swapchain extent, HiDPI factor, and user zoom.
    #[allow(clippy::cast_possible_truncation)]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            let physical = Size::new(config.width as f32, config.height as f32);
            // The kit's standard HiDPI mapping (device scale + logical viewport)
            // in one call — no hand-rolled viewport arithmetic here.
            self.runtime.set_surface(physical, self.effective_scale());
        }
    }

    /// Handles a Ctrl+zoom shortcut, returning `true` when `key` was a zoom chord.
    fn handle_zoom_shortcut(&mut self, key: &Key) -> bool {
        let Key::Character(text) = key else {
            return false;
        };
        let new_zoom = match text.as_str() {
            "=" | "+" => Some((self.zoom * 1.1).min(5.0)),
            "-" => Some((self.zoom / 1.1).max(0.3)),
            "0" => Some(1.0),
            _ => None,
        };
        let Some(new_zoom) = new_zoom else {
            return false;
        };
        self.zoom = new_zoom;
        self.sync_viewport();
        self.runtime.request_frame(FrameReason::WindowChange);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        true
    }

    /// Renders one frame: run a scheduler-gated reactive frame (rebuilding the view
    /// from the current state when one is due), read the committed scene, and
    /// present it on-screen via Vello.
    fn render_frame(&mut self) {
        let cpu_start = Instant::now();
        self.runtime.run_due_frame_reactive();
        let snapshot = self.runtime.snapshot();
        let cpu = cpu_start.elapsed();

        let (Some(surface), Some(renderer)) = (self.surface.as_ref(), self.renderer.as_mut())
        else {
            return;
        };

        let physical_size = Size::new(
            surface.config().width as f32,
            surface.config().height as f32,
        );
        let render_frame = RenderFrame::new(snapshot, physical_size);
        let submission = FrameSubmission::new(render_frame);

        let present_start = Instant::now();
        match renderer.present_to_surface(&submission, surface) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("present failed: {error:?}");
                if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }
        let present = present_start.elapsed();

        if let Some(report) = self.profiler.record(cpu, present) {
            println!("[perf] {report}");
            // Building a live source walks + formats the whole tree, so only
            // pay for it while a DevTools client is actually attached.
            if self.inspect.has_client() {
                self.inspect
                    .set(live_source(&self.runtime, self.profiler.stats()));
            }
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Binds and serves a loopback, token-authed DevTools endpoint on a background
    /// thread, so an external `obscur-devtools` client can inspect this live
    /// window's real controls (tree/layout/styles) + frame perf (§7.6/§7.9).
    fn start_devtools_endpoint(&self) {
        let listener = match SocketListener::bind_loopback() {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("[devtools] endpoint disabled (bind failed: {error})");
                return;
            }
        };
        if let Ok(Some(addr)) = listener.local_addr() {
            println!(
                "[devtools] live endpoint on {addr}\n\
                 [devtools] try: obscur-devtools inspect all --tcp {addr} --token dev"
            );
        }
        let shared = self.inspect.clone();
        thread::spawn(move || devtools_serve(&listener, &shared));
    }
}

/// Serves the DevTools endpoint over `listener` forever, one session at a time,
/// each answering inspects from the shared, per-frame-refreshed `source`.
fn devtools_serve(listener: &SocketListener, source: &SharedInspectSource) {
    while let Ok(stream) = listener.accept() {
        let mut authenticator = Authenticator::new();
        authenticator.register_token("dev", ScopeSet::all_read());
        let mut endpoint = DevToolsEndpoint::new(authenticator, RuntimeBridge::new(source.clone()));
        endpoint.enable();
        source.client_connected();
        let _ = SocketTransport::new(stream).serve(&mut endpoint);
        source.client_disconnected();
    }
}

impl ApplicationHandler for WireframeWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — interactive Wireframe Kit (Vello)")
            .with_inner_size(LogicalSize::new(
                f64::from(INITIAL_SIZE.width),
                f64::from(INITIAL_SIZE.height),
            ));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };
        window.set_ime_allowed(true);

        match init_gpu(&window) {
            Ok((surface, config, device, queue)) => {
                let vello_surface = VelloSurface::new(surface, config);
                vello_surface.configure(&device);
                match VelloDevice::new(device.clone(), queue) {
                    Ok(vello_device) => {
                        let mut renderer =
                            VelloRenderer::new(RendererId::from_raw(2)).with_device(vello_device);
                        if let Some(primary_bytes) = load_system_font() {
                            wire_text_stack(&mut self.runtime, &mut renderer, primary_bytes);
                        } else {
                            eprintln!("no system font found; text will not render");
                        }
                        self.scale_factor = window.scale_factor();
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        self.sync_viewport();
                        self.runtime.request_frame(FrameReason::UiCommit);
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                        println!(
                            "wireframe window ready — hover/click the bare controls; their look is the UA default."
                        );
                        self.start_devtools_endpoint();
                    }
                    Err(error) => {
                        eprintln!("failed to build the Vello device: {error}");
                        event_loop.exit();
                    }
                }
            }
            Err(error) => {
                eprintln!("no usable GPU adapter/device/surface: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device)) = (self.surface.as_mut(), self.device.as_ref())
                {
                    surface.resize(device, new_size.width, new_size.height);
                }
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
                self.runtime.handle_pointer(moved);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                let event = self.pointer_button_event(state);
                let dispatch = self.runtime.handle_pointer(event);
                if let Some(command) = dispatch.command() {
                    println!("[click] resolved command {command:?}");
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Route the wheel to the REAL `overflow: auto` scroll container under
                // the pointer (the `#app` page): `handle_scroll` hit-tests the
                // cursor, walks to the innermost scroll container, and adjusts its
                // clamped `scroll_top` (CSS Overflow 3 §scroll-container). The wheel
                // delta is physical px; divide by the effective scale for logical px.
                // A positive wheel `lines` scrolls the content up (toward the top), so
                // the scroll delta is negated.
                let step_px = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => lines * WHEEL_LINE_STEP,
                    #[allow(clippy::cast_possible_truncation)]
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                #[allow(clippy::cast_possible_truncation)]
                let cursor = Point::new(self.cursor.0 as f32, self.cursor.1 as f32);
                self.runtime
                    .handle_scroll(-step_px / self.effective_scale(), cursor);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && self.modifiers.control_key()
                    && self.handle_zoom_shortcut(&event.logical_key)
                {
                    return;
                }
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        self.runtime
                            .handle_key(KeyEvent::new(physical, logical, pressed));
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                let event = match ime {
                    Ime::Enabled => Some(CompositionEvent::Start),
                    Ime::Preedit(text, cursor) => {
                        let mut preedit = Preedit::new(text);
                        if let Some((start, _end)) = cursor {
                            preedit = preedit.with_caret(start);
                        }
                        Some(CompositionEvent::Update(preedit))
                    }
                    Ime::Commit(text) => Some(CompositionEvent::Commit(text.into())),
                    Ime::Disabled => Some(CompositionEvent::Cancel),
                };
                if let Some(event) = event {
                    self.runtime.handle_ime(event);
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

/// A short word for a tri-state checkbox's current value.
fn tri_word(tri: &obscur_kit::widgets::checkbox::Checkbox) -> &'static str {
    if tri.is_indeterminate() {
        "mixed"
    } else if tri.is_checked() {
        "checked"
    } else {
        "unchecked"
    }
}

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Initialises a wgpu swapchain surface + device/queue for on-screen rendering.
///
/// # Errors
///
/// Returns an error if the surface cannot be created, no compatible GPU adapter is
/// available (as in a headless environment), the device cannot be created, or the
/// surface exposes no supported default configuration.
fn init_gpu(
    window: &Arc<Window>,
) -> Result<
    (
        wgpu::Surface<'static>,
        wgpu::SurfaceConfiguration,
        wgpu::Device,
        wgpu::Queue,
    ),
    Box<dyn std::error::Error>,
> {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window.clone())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    let size = window.inner_size();
    let config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or("surface unsupported by adapter")?;
    Ok((surface, config, device, queue))
}

/// Wires the multi-font fallback stack onto the runtime and the Vello renderer.
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, primary_bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();
    let primary_id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        primary_bytes.clone(),
    );
    renderer.register_font(font_handle(primary_id), primary_bytes);

    if let Some(fallback_bytes) = load_fallback_font() {
        let fallback_id = registry.register(
            FontFamily::generic(GenericFamily::SansSerif),
            FontAttributes::default(),
            fallback_bytes.clone(),
        );
        renderer.register_font(font_handle(fallback_id), fallback_bytes);
    }

    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

/// The scene font handle (a `u32`) for a resolved [`FontId`].
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes for the text path (best effort).
fn load_system_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/Hack-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/Arimo-Regular.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}

/// Loads a second, CJK-covering fallback font's bytes (best effort).
fn load_fallback_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto/NotoSansCJKsc-Regular.otf",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
    ];
    CANDIDATES.iter().find_map(|p| std::fs::read(p).ok())
}

/// Builds the winit event loop and runs the application to completion.
///
/// # Errors
///
/// Propagates any event-loop construction or run error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit interactive Wireframe Kit → Vello (windowed) ==");
    println!("(bare controls styled ONLY by the installed UA default stylesheet)\n");

    let event_loop = EventLoop::new()?;
    let mut app = WireframeWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

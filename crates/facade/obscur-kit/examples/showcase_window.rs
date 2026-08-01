#![doc = include_str!("showcase_window.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::devtools::{
    DiagnosticEventSink, RuntimeEventPublisher, SharedInspectSource, live_source,
};
use obscur_kit::devtools_protocol::message::ScopeSet;
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::bridge::RuntimeBridge;
use obscur_kit::devtools_server::endpoint::DevToolsEndpoint;
use obscur_kit::devtools_server::transport::{EventQueue, SocketListener, SocketTransport};
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
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
use winit::keyboard::Key;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "showcase/scene.rs"]
mod showcase;

use showcase::ShowcaseState;

#[doc = include_str!("showcase_window.doc/CMD_BUTTON.md")]
const CMD_BUTTON: InternedSymbol = InternedSymbol::from_raw(0x5001);
const CMD_CHECKBOX: InternedSymbol = InternedSymbol::from_raw(0x5002);
const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x5003);

#[doc = include_str!("showcase_window.doc/WHEEL_LINE_STEP.md")]
const WHEEL_LINE_STEP: f32 = 64.0;

#[doc = include_str!("showcase_window.doc/INITIAL_SIZE.md")]
const INITIAL_SIZE: Size = Size::new(1000.0, 700.0);

#[doc = include_str!("showcase_window.doc/ShowcaseWindow.md")]
struct ShowcaseWindow {
    #[doc = include_str!("showcase_window.doc/window.md")]
    window: Option<Arc<Window>>,
    #[doc = include_str!("showcase_window.doc/surface.md")]
    surface: Option<VelloSurface>,
    #[doc = include_str!("showcase_window.doc/device.md")]
    device: Option<wgpu::Device>,
    #[doc = include_str!("showcase_window.doc/renderer.md")]
    renderer: Option<VelloRenderer>,
    #[doc = include_str!("showcase_window.doc/runtime.md")]
    runtime: UiRuntime,
    #[doc = include_str!("showcase_window.doc/cursor.md")]
    cursor: (f64, f64),
    #[doc = include_str!("showcase_window.doc/scale_factor.md")]
    scale_factor: f64,
    #[doc = include_str!("showcase_window.doc/zoom.md")]
    zoom: f32,
    #[doc = include_str!("showcase_window.doc/scroll_y.md")]
    scroll_y: f32,
    #[doc = include_str!("showcase_window.doc/modifiers.md")]
    modifiers: ModifiersState,
    #[doc = include_str!("showcase_window.doc/last_inspect_node.md")]
    last_inspect_node: Option<u64>,
    #[doc = include_str!("showcase_window.doc/profiler.md")]
    profiler: FrameProfiler,
    #[doc = include_str!("showcase_window.doc/inspect.md")]
    inspect: SharedInspectSource,
    #[doc = include_str!("showcase_window.doc/events.md")]
    events: EventQueue,
    #[doc = include_str!("showcase_window.doc/publisher.md")]
    publisher: RuntimeEventPublisher,
}

impl ShowcaseWindow {
    #[doc = include_str!("showcase_window.doc/new.md")]
    fn new(asset_dir: &std::path::Path) -> Self {
        let state = Rc::new(RefCell::new(ShowcaseState::default()));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);

        // Real filesystem image loader + decoder, rooted at the staged sandbox so
        // `background-image: url("gallery.png")` and the `<img>` primitive resolve
        // to decoded pixels (same as `showcase_png.rs`).
        KitBootstrap::new().install_image_resources(&mut runtime, asset_dir);

        // Interactive wiring through the runtime's real command + reactive-render
        // path (no geometric hacks):
        //  - the Button's click resolves to CMD_BUTTON → bump the counter;
        //  - the Checkbox's change resolves to CMD_CHECKBOX → toggle checked;
        //  - the Input's per-keystroke edit fires CMD_INPUT with the resulting
        //    text → mirror it into `state.input_text`;
        //  - a view BUILDER rebuilds `showcase::view(&state)` each reactive frame;
        //  - the compiled showcase stylesheet is what the reactive path matches.
        {
            let s = state.clone();
            runtime.on_command(CMD_BUTTON, move || {
                let mut st = s.borrow_mut();
                st.counter += 1;
                println!("[button] counter = {}", st.counter);
            });
        }
        {
            let s = state.clone();
            runtime.on_command(CMD_CHECKBOX, move || {
                let mut st = s.borrow_mut();
                st.checked = !st.checked;
                println!("[checkbox] checked = {}", st.checked);
            });
        }
        {
            let s = state.clone();
            runtime.on_text_input(CMD_INPUT, move |text| {
                text.clone_into(&mut s.borrow_mut().input_text);
                println!("[input] value = {text:?}");
            });
        }
        {
            let s = state.clone();
            runtime.set_root_view(move || showcase::view(&s.borrow()));
        }
        runtime.set_stylesheets(vec![compile(showcase::stylesheet())]);

        // DevTools event push: the runtime reports its component failures onto
        // the queue as they happen, and `publisher` adds the frame/focus
        // transitions each frame. Both are producers for a subscription — without
        // one, the endpoint would (correctly) refuse to grant any.
        let events = EventQueue::new();
        runtime.set_diagnostic_sink(Box::new(DiagnosticEventSink::new(events.clone())));
        let publisher = RuntimeEventPublisher::attach(&runtime, events.clone());

        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            zoom: 1.0,
            scroll_y: 0.0,
            modifiers: ModifiersState::empty(),
            last_inspect_node: None,
            profiler: FrameProfiler::new(),
            inspect: SharedInspectSource::new(),
            events,
            publisher,
        }
    }

    #[doc = include_str!("showcase_window.doc/base_scale.md")]
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    #[doc = include_str!("showcase_window.doc/effective_scale.md")]
    fn effective_scale(&self) -> f32 {
        self.base_scale() * self.zoom
    }

    #[doc = include_str!("showcase_window.doc/content_height_logical.md")]
    fn content_height_logical(&self) -> f32 {
        self.runtime
            .inspect_snapshot()
            .nodes
            .iter()
            .filter_map(|node| node.bounds)
            .map(|rect| rect.origin.y + rect.size.height)
            .fold(0.0_f32, f32::max)
    }

    #[doc = include_str!("showcase_window.doc/max_scroll_y.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn max_scroll_y(&self) -> f32 {
        let Some(surface) = self.surface.as_ref() else {
            return 0.0;
        };
        let physical_height = surface.config().height as f32;
        let content_height = self.content_height_logical() * self.effective_scale();
        (content_height - physical_height).max(0.0)
    }

    #[doc = include_str!("showcase_window.doc/apply_scroll.md")]
    fn apply_scroll(&mut self) {
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
        let offset = self.scroll_y / self.effective_scale();
        self.runtime.set_content_offset(0.0, offset);
    }

    #[doc = include_str!("showcase_window.doc/log_scale_sanity.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn log_scale_sanity(&self) {
        if let Some(surface) = self.surface.as_ref() {
            let (pw, ph) = (surface.config().width, surface.config().height);
            let scale = self.effective_scale();
            println!(
                "[scale] window {pw}×{ph} px · device_scale {:.3} · zoom {:.2} · effective_scale {:.3} · viewport {:.0}×{:.0} logical (reflows on resize) · max_scroll {:.0} px",
                self.base_scale(),
                self.zoom,
                scale,
                pw as f32 / scale,
                ph as f32 / scale,
                self.max_scroll_y(),
            );
        }
    }

    #[doc = include_str!("showcase_window.doc/pointer_button_event.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
    }

    #[doc = include_str!("showcase_window.doc/sync_viewport.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            let physical = Size::new(config.width as f32, config.height as f32);
            // The kit's standard HiDPI mapping (device scale + logical viewport)
            // in one call — no hand-rolled viewport arithmetic here.
            self.runtime.set_surface(physical, self.effective_scale());
        }
        // The scale/zoom just changed, so the scrollable range may have too —
        // re-clamp and re-push the content offset.
        self.apply_scroll();
    }

    #[doc = include_str!("showcase_window.doc/handle_zoom_shortcut.md")]
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
        // Ctrl+0 resets zoom *and* scroll to the top.
        if text.as_str() == "0" {
            self.scroll_y = 0.0;
        }
        self.zoom = new_zoom;
        self.sync_viewport();
        self.runtime.request_frame(FrameReason::WindowChange);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        println!(
            "[zoom] {:.0}% (device scale {:.2})",
            self.zoom * 100.0,
            self.effective_scale()
        );
        true
    }

    #[doc = include_str!("showcase_window.doc/render_frame.md")]
    fn render_frame(&mut self) {
        // Run a REACTIVE frame only if one is due (§3.5); the runtime holds the
        // view builder + stylesheets, so it rebuilds `showcase::view` from the
        // current state itself. `None` means nothing was due — we still re-present
        // the last committed snapshot so a bare expose repaints rather than blanks.
        // Run the due frame for its side effects; presents are silent — activity
        // is logged as EVENTS (clicks, input, resize), not once per frame.
        // Time the CPU phase (reactive build → reconcile → style → layout →
        // compose) separately from the GPU present so `[perf]` breaks the two out.
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

        // Present on-screen (offscreen render + blit-to-surface, the documented
        // Vello 0.9 pattern the counter example uses).
        let present_start = Instant::now();
        match renderer.present_to_surface(&submission, surface) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("present failed: {error:?}");
                // A lost/outdated swapchain is recoverable: reconfigure, skip.
                if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }
        let present = present_start.elapsed();

        // Feed the kit profiler; once per second it returns a report line and we
        // publish the live state (tree/layout/styles + Rendering=perf) to any
        // connected DevTools client. A redraw is requested every frame so the
        // present rate (the `[perf]` FPS) is real rather than event-driven.
        // Frame/focus events for any subscriber. Gated on an attached client for
        // the same reason the live source is: at 60fps an unwatched host would
        // otherwise churn the bounded queue every frame for nobody.
        if self.inspect.has_client() {
            self.publisher.publish(&self.runtime);
        }
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

    #[doc = include_str!("showcase_window.doc/start_devtools_endpoint.md")]
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
                 [devtools] try: obscur-devtools inspect rendering --tcp {addr} --token dev   (or: tree | layout | all)\n\
                 [devtools] or:  obscur-devtools subscribe frames --tcp {addr} --token dev   (or: focus | diagnostics)"
            );
        }
        let shared = self.inspect.clone();
        let events = self.events.clone();
        thread::spawn(move || devtools_serve(&listener, &shared, &events));
    }
}

#[doc = include_str!("showcase_window.doc/devtools_serve.md")]
fn devtools_serve(listener: &SocketListener, source: &SharedInspectSource, events: &EventQueue) {
    while let Ok(stream) = listener.accept() {
        let mut authenticator = Authenticator::new();
        authenticator.register_token("dev", ScopeSet::observer());
        let mut endpoint = DevToolsEndpoint::new(authenticator, RuntimeBridge::new(source.clone()));
        endpoint.enable();
        source.client_connected();
        // A client-side fault ends that session, not the server. The 20ms poll
        // bounds how long a queued event waits, and is what the thread sleeps in
        // between wakes — it does not spin.
        let _ = SocketTransport::new(stream).serve_with_events(
            &mut endpoint,
            events,
            Duration::from_millis(20),
        );
        source.client_disconnected();
    }
}

impl ApplicationHandler for ShowcaseWindow {
    #[doc = include_str!("showcase_window.doc/resumed.md")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — capability showcase (Vello)")
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
        // Let the platform IME deliver composition events for the editable Input.
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
                        // Seed the HiDPI factor from the freshly created window so
                        // the very first frame lays out in logical px and renders
                        // at physical resolution (no initial "tiny" frame).
                        self.scale_factor = window.scale_factor();
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        self.sync_viewport();
                        self.log_scale_sanity();
                        self.runtime.request_frame(FrameReason::UiCommit);
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                        self.start_devtools_endpoint();
                    }
                    Err(error) => {
                        eprintln!("failed to build the Vello device: {error}");
                        event_loop.exit();
                    }
                }
            }
            Err(error) => {
                // Expected in a headless environment (no adapter). Not a panic.
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
                // Real domain path: winit → `PointerConverter::moved` →
                // `handle_pointer` → hit-test → hover / drag-selection.
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
                self.runtime.handle_pointer(moved);
                // In-window inspector: print the hovered element's readout once per
                // hover change so the console names what the highlight is tracing.
                if self.runtime.inspect_mode() {
                    let info = self.runtime.hovered_inspect();
                    let node = info.as_ref().map(|i| i.node.raw());
                    if node != self.last_inspect_node {
                        self.last_inspect_node = node;
                        if let Some(info) = info {
                            println!("[inspect] {info}");
                        }
                    }
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                // Real domain pointer path for both transitions: `Down` anchors
                // the click and drives focus (seating the caret when it lands on
                // the Input), the matching `Up` completes it — `handle_pointer`
                // resolves a widget activation to CMD_BUTTON / CMD_CHECKBOX and
                // runs its handler, mutating the shared state.
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
                // A wheel notch scrolls the fit-to-width content vertically. Winit
                // reports either whole lines or raw pixels; normalize both to a
                // physical-px step, accumulate into `scroll_y` (down = positive =
                // reveal content further down), clamp, and push the logical offset.
                let step_px = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => lines * WHEEL_LINE_STEP,
                    #[allow(clippy::cast_possible_truncation)]
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                self.scroll_y -= step_px;
                self.apply_scroll();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
                println!(
                    "[scroll] y = {:.0} px (max {:.0})",
                    self.scroll_y,
                    self.max_scroll_y()
                );
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                // Track the live modifier chord so the Ctrl+zoom shortcuts below
                // can be recognised (and swallowed) before text reaches the field.
                self.modifiers = modifiers.state();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // The window moved to a display with a different HiDPI ratio: keep
                // layout in logical px and re-scale. A `Resized` with the new
                // physical extent typically follows and re-syncs again.
                self.scale_factor = scale_factor;
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // F12 toggles the in-window inspect overlay (spec §7.6) — handled
                // before forwarding so it never reaches a focused Input as text.
                if event.state == ElementState::Pressed
                    && event.physical_key == WinitPhysicalKey::Code(KeyCode::F12)
                {
                    let on = !self.runtime.inspect_mode();
                    self.runtime.set_inspect_mode(on);
                    self.last_inspect_node = None;
                    println!(
                        "[inspect] {}",
                        if on {
                            "ON — hover elements to highlight them; F12 to toggle off"
                        } else {
                            "OFF"
                        }
                    );
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                    return;
                }
                // Ctrl+zoom shortcuts are handled here FIRST (on key-down only),
                // ahead of forwarding to the runtime, so a zoom chord adjusts the
                // effective device scale instead of being typed into the Input.
                if event.state == ElementState::Pressed
                    && self.modifiers.control_key()
                    && self.handle_zoom_shortcut(&event.logical_key)
                {
                    return;
                }
                // Real domain keyboard path: winit → `KeyboardConverter` →
                // `handle_key`. When the Input is focused, mutating keys drive its
                // editing session, whose `on_text_input` handler mirrors the text
                // into the shared state.
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
                // Real IME composition path onto the focused editable (spec §5.12).
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

#[doc = include_str!("showcase_window.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("showcase_window.doc/stage_image_asset.md")]
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-showcase-win-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(
        dir.join(showcase::IMAGE_ASSET_NAME),
        showcase::image_asset_bytes(),
    )
    .expect("write image asset");
    dir
}

#[doc = include_str!("showcase_window.doc/init_gpu.md")]
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

#[doc = include_str!("showcase_window.doc/wire_text_stack.md")]
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

#[doc = include_str!("showcase_window.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("showcase_window.doc/load_system_font.md")]
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

#[doc = include_str!("showcase_window.doc/load_fallback_font.md")]
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

#[doc = include_str!("showcase_window.doc/main.md")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit capability showcase → Vello (windowed, interactive) ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)\n");

    let asset_dir = stage_image_asset();
    let event_loop = EventLoop::new()?;
    let mut app = ShowcaseWindow::new(&asset_dir);
    event_loop.run_app(&mut app)?;
    Ok(())
}

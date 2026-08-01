#![doc = include_str!("windowed_counter.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, GenericFamily};
use obscur_kit::text::ime::{CompositionEvent, Preedit};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use obscur_kit::window_winit::backend::{UserEvent, WinitAppHandler, WinitWindowBackend};
use obscur_kit::window_winit::conversion::{KeyboardConverter, PointerConverter};
use obscur_kit::windowing::port::WindowBackend;
use obscur_kit::windowing::request::{CloseWindow, CreateWindow};
use obscur_kit::windowing::window::{WindowEvent as DomainWindowEvent, WindowProperties};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, Ime, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey as WinitPhysicalKey;
use winit::window::{Window, WindowId};

#[doc = include_str!("windowed_counter.doc/CMD_DECREMENT.md")]
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

#[doc = include_str!("windowed_counter.doc/WindowedCounter.md")]
struct WindowedCounter {
    #[doc = include_str!("windowed_counter.doc/backend.md")]
    backend: WinitWindowBackend,
    #[doc = include_str!("windowed_counter.doc/handler.md")]
    handler: WinitAppHandler,
    #[doc = include_str!("windowed_counter.doc/window_id.md")]
    window_id: obscur_kit::identity::WindowId,
    #[doc = include_str!("windowed_counter.doc/window.md")]
    window: Option<Arc<Window>>,
    #[doc = include_str!("windowed_counter.doc/surface.md")]
    surface: Option<VelloSurface>,
    #[doc = include_str!("windowed_counter.doc/device.md")]
    device: Option<wgpu::Device>,
    #[doc = include_str!("windowed_counter.doc/renderer.md")]
    renderer: Option<VelloRenderer>,
    #[doc = include_str!("windowed_counter.doc/runtime.md")]
    runtime: UiRuntime,
    #[doc = include_str!("windowed_counter.doc/count.md")]
    count: Rc<Cell<i32>>,
    #[doc = include_str!("windowed_counter.doc/cursor.md")]
    cursor: (f64, f64),
}

impl WindowedCounter {
    #[doc = include_str!("windowed_counter.doc/new.md")]
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let count = Rc::new(Cell::new(0));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(480.0, 240.0));

        // Wire the interactive counter through the runtime's real command +
        // reactive-render path (no geometric hack):
        //  - register a handler per button command that mutates the shared count;
        //  - install a view BUILDER that rebuilds the counter view from the
        //    current count each reactive frame;
        //  - install the stylesheet the reactive path matches against.
        // A click on a button then resolves (via the reserved `EVENT_CLICK`
        // symbol the button binds) to its command, runs the handler, and the next
        // reactive frame rebuilds the view with the new value.
        {
            let c = count.clone();
            runtime.on_command(CMD_INCREMENT, move || c.set(c.get() + 1));
        }
        {
            let c = count.clone();
            runtime.on_command(CMD_DECREMENT, move || c.set(c.get() - 1));
        }
        {
            let c = count.clone();
            runtime.set_root_view(move || build_counter_view(c.get()));
        }
        runtime.set_stylesheets(vec![counter_stylesheet()]);

        // Create the window THROUGH THE PORT: domain properties in, a `WindowId`
        // out, no winit type named. The id is live immediately; the OS window it
        // names is built by `handler` on the first event-loop turn (deferred
        // creation), which is why `window`/`surface` stay `None` until `resumed`.
        let mut backend = WinitWindowBackend::new();
        let window_id = backend.create_window(CreateWindow::new(
            WindowProperties::default()
                .with_title("obscur-kit — windowed counter")
                .with_size(Size::new(480.0, 240.0)),
        ))?;
        let handler = backend.app_handler();

        Ok(Self {
            backend,
            handler,
            window_id,
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            count,
            cursor: (0.0, 0.0),
        })
    }

    #[doc = include_str!("windowed_counter.doc/pointer_button_event.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
    }

    #[doc = include_str!("windowed_counter.doc/sync_viewport.md")]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            let viewport = Size::new(config.width as f32, config.height as f32);
            self.runtime.set_viewport(viewport);
        }
    }

    #[doc = include_str!("windowed_counter.doc/drain_window_events.md")]
    fn drain_window_events(&mut self, event_loop: &ActiveEventLoop) {
        for (id, event) in self.backend.poll_window_events() {
            match event {
                DomainWindowEvent::Resized(size) => {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let (width, height) = (size.width as u32, size.height as u32);
                    if let (Some(surface), Some(device)) =
                        (self.surface.as_mut(), self.device.as_ref())
                    {
                        surface.resize(device, width, height);
                    }
                    // Keep the layout viewport in step with the new extent (the
                    // surface/device borrows above have ended) and schedule a
                    // frame so the next redraw re-lays-out under it.
                    self.sync_viewport();
                    self.runtime.request_frame(FrameReason::WindowChange);
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
                DomainWindowEvent::CloseRequested => {
                    // We honor it: close through the port, then leave.
                    let result = self.backend.close_window(CloseWindow::request(id));
                    println!("[window] close requested -> port answered {result:?}");
                    event_loop.exit();
                }
                _ => {}
            }
        }
    }

    #[doc = include_str!("windowed_counter.doc/render_frame.md")]
    fn render_frame(&mut self) {
        // --- Stage 1+2: run a REACTIVE frame only if one is due (§3.5). -------
        // The runtime holds the view BUILDER (installed in `new`) and its
        // stylesheets, so it rebuilds the counter view from the current `count`
        // itself — no `&View` is threaded here. First paint, input (hover/focus),
        // resize, and each click-dispatched command `request_frame` a reason;
        // `run_due_frame_reactive` produces at most one frame per redraw and
        // returns `None` when nothing is pending. We still present the (unchanged)
        // snapshot on a `None` so a bare `RedrawRequested` — e.g. an OS expose —
        // repaints the last committed scene rather than a blank.
        let produced = self.runtime.run_due_frame_reactive();
        let snapshot = self.runtime.snapshot();

        // Nothing to present until the surface and renderer exist.
        let (Some(surface), Some(renderer)) = (self.surface.as_ref(), self.renderer.as_mut())
        else {
            return;
        };

        // --- Stage 4: package the snapshot into a submission for the port. ----
        // The physical viewport is the surface's configured swapchain extent.
        // u32 -> f32 for window extents is intentional and lossless in practice
        // (cast_precision_loss is allowed workspace-wide).
        let physical_size = Size::new(
            surface.config().width as f32,
            surface.config().height as f32,
        );
        let render_frame = RenderFrame::new(snapshot, physical_size);
        let submission = FrameSubmission::new(render_frame);

        // --- Stage 5: present on-screen (offscreen render + blit-to-surface). -
        match renderer.present_to_surface(&submission, surface) {
            Ok(result) => {
                // `produced` distinguishes a freshly run frame from a re-present
                // of the last committed scene (scheduler had nothing due).
                let frame_note = produced.as_ref().map_or_else(
                    || "re-present (no new frame)".to_owned(),
                    |f| format!("frame id={} composed={}", f.id().raw(), f.composed()),
                );
                println!(
                    "present {}: {frame_note} | vello -> {result:?} (presented)",
                    renderer.frames_rendered(),
                );
            }
            Err(error) => {
                eprintln!("present failed: {error:?}");
                // A lost/outdated swapchain is recoverable: reconfigure and skip
                // this frame. The next redraw acquires a fresh texture.
                if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }

        // Redraw on demand only (input / resize): the window is idle when
        // nothing changes rather than spinning the GPU.
    }
}

impl ApplicationHandler<UserEvent> for WindowedCounter {
    #[doc = include_str!("windowed_counter.doc/resumed.md")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Let the backend's handler service the loop turn: it builds the window
        // queued by `create_window` and enumerates the displays.
        self.handler.resumed(event_loop);

        // `resumed` can fire more than once (e.g. on Android); build only once.
        if self.window.is_some() {
            return;
        }

        // 1. Take the live platform handle for the id the port minted for us.
        //    `None` here means the backend could not materialize the window;
        //    it records the reason rather than panicking.
        let Some(window) = self.backend.native_window(self.window_id) else {
            eprintln!("the window backend did not materialize a window");
            event_loop.exit();
            return;
        };
        // Allow the platform IME to deliver composition events for this window
        // (spec §5.12). A real app toggles this per focused editable; the counter
        // has no text field, so it is enabled once here — the `WindowEvent::Ime`
        // arm below then drives `UiRuntime::handle_ime` end-to-end. IME is not a
        // §6 window property, so it is set on the platform handle directly.
        window.set_ime_allowed(true);

        // 2. Initialise wgpu + the swapchain surface and build the Vello
        //    renderer on top of the host-provided device/queue.
        match init_gpu(&window) {
            Ok((surface, config, device, queue)) => {
                let vello_surface = VelloSurface::new(surface, config);
                vello_surface.configure(&device);
                match VelloDevice::new(device.clone(), queue) {
                    Ok(vello_device) => {
                        let mut renderer =
                            VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
                        // Text is a host resource (like the GPU device). Wire the
                        // FULL multi-font fallback stack: the runtime itemizes
                        // each label into uniform bidi+script runs
                        // (`UnicodeTextItemizer`), resolves each run to the first
                        // registered font whose `cmap` COVERS it (`FontRegistry`),
                        // and shapes it against that resolved face's in-memory
                        // bytes (font-less `HarfrustShaper`). The renderer gets the
                        // SAME id->bytes assignment so scene font handles line up
                        // with the fonts it rasterizes.
                        if let Some(primary_bytes) = load_system_font() {
                            wire_text_stack(&mut self.runtime, &mut renderer, primary_bytes);
                        } else {
                            eprintln!("no system font found; text will not render");
                        }
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        // Lay out at the surface's physical extent, not the
                        // placeholder viewport, so the scene fills the window.
                        self.sync_viewport();
                        // Schedule the first frame so the initial `run_due_frame`
                        // has something due, then ask winit to redraw.
                        self.runtime.request_frame(FrameReason::UiCommit);
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                    Err(error) => {
                        eprintln!("failed to build the Vello device: {error}");
                        event_loop.exit();
                    }
                }
            }
            Err(error) => {
                // Expected in a headless CI environment (no adapter). On a real
                // machine an adapter/device is present.
                eprintln!("no usable GPU adapter/device/surface: {error}");
                event_loop.exit();
            }
        }
    }

    #[doc = include_str!("windowed_counter.doc/window_event.md")]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Forward first: this is what buffers the domain lifecycle events that
        // `drain_window_events` reads on the next `about_to_wait`.
        self.handler
            .window_event(event_loop, window_id, event.clone());

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                // Track the raw position for the geometric click regions, then
                // feed the *real* domain path: winit → `PointerConverter::moved`
                // → `handle_pointer` → hit-test → hover → `:hover` restyle. A
                // hover change schedules an `Input` frame inside the runtime; we
                // still ask winit to redraw so that frame is actually drained.
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
                self.runtime.handle_pointer(moved);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                // Feed the *real* domain pointer path for BOTH transitions: a
                // `Down` at the cursor anchors the click and drives focus; the
                // matching `Up` on the same node completes the click, and
                // `handle_pointer` resolves it (via the button's reserved
                // `EVENT_CLICK` binding) to `CMD_INCREMENT`/`CMD_DECREMENT` and
                // runs the registered handler, which mutates the shared count.
                // The dispatch itself requests a frame; the next redraw's
                // `run_due_frame_reactive` rebuilds the view with the new value.
                let event = self.pointer_button_event(state);
                let dispatch = self.runtime.handle_pointer(event);
                if let Some(command) = dispatch.command() {
                    println!(
                        "[click] resolved command {command:?} -> count = {}",
                        self.count.get()
                    );
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Feed the *real* domain keyboard path, mirroring `CursorMoved`
                // above: winit → `KeyboardConverter` → `handle_key` → focused
                // node → command resolution. The converter splits the winit
                // `KeyEvent` into its layout-independent physical key and its
                // layout-dependent logical key; only a modeled physical position
                // produces a domain event (media/IME keys convert to `None`).
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        let dispatch = self
                            .runtime
                            .handle_key(KeyEvent::new(physical, logical, pressed));
                        if let Some(command) = dispatch.command() {
                            println!("[key] resolved command: {command:?}");
                        }
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                // Feed the *real* IME composition path: winit's `Ime` events map
                // 1:1 onto the domain `CompositionEvent`s the runtime consumes
                // (spec §5.12). `Enabled` starts a composition, `Preedit` carries
                // the uncommitted (shown-but-not-inserted) preedit, `Commit`
                // finalizes the composed text into the focused editable as one
                // edit, and `Disabled` cancels. `handle_ime` is a no-op when no
                // editable is focused. The winit preedit cursor is a byte range
                // within the preedit; its start seats the display caret.
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
            // Close is NOT handled here: the handler converted it into a domain
            // `CloseRequested`, and `drain_window_events` decides what to do with
            // it — which is what §6.1 means by "the application decides whether to
            // honor it".
            _ => {}
        }
    }

    #[doc = include_str!("windowed_counter.doc/about_to_wait.md")]
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handler.about_to_wait(event_loop);
        self.drain_window_events(event_loop);
    }

    #[doc = include_str!("windowed_counter.doc/user_event.md")]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        self.handler.user_event(event_loop, event);
    }
}

#[doc = include_str!("windowed_counter.doc/init_gpu.md")]
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
    // wgpu 29: `Instance::default()` selects all compiled-in backends.
    let instance = wgpu::Instance::default();
    // `Arc<Window>` implements `Into<SurfaceTarget<'static>>`, so the surface is
    // `'static` and can outlive individual frames.
    let surface = instance.create_surface(window.clone())?;
    // Request an adapter compatible with the surface we intend to present to.
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))?;
    // wgpu 29: `request_device` takes a single `&DeviceDescriptor` and yields
    // `Result<(Device, Queue), RequestDeviceError>`.
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    // Derive the surface configuration from the current window size (clamped to
    // >= 1 so wgpu accepts it). The immutable borrows of `surface` above all end
    // before it is moved out to the caller.
    let size = window.inner_size();
    let config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or("surface unsupported by adapter")?;
    Ok((surface, config, device, queue))
}

#[doc = include_str!("windowed_counter.doc/load_system_font.md")]
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

#[doc = include_str!("windowed_counter.doc/load_fallback_font.md")]
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

#[doc = include_str!("windowed_counter.doc/wire_text_stack.md")]
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, primary_bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();

    // Primary under the SansSerif generic family -> id 0.
    let primary_id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        primary_bytes.clone(),
    );
    renderer.register_font(font_handle(primary_id), primary_bytes);

    // Fallback under the SAME family -> id 1 (coverage, not name, disambiguates).
    if let Some(fallback_bytes) = load_fallback_font() {
        let fallback_id = registry.register(
            FontFamily::generic(GenericFamily::SansSerif),
            FontAttributes::default(),
            fallback_bytes.clone(),
        );
        renderer.register_font(font_handle(fallback_id), fallback_bytes);
    } else {
        // No second font: the primary alone can't cover the CJK part of the
        // demo label, so those glyphs paint as .notdef. Registering a covering
        // face above is what activates the fallback path.
        eprintln!(
            "no fallback font found; mixed-script glyphs the primary lacks render as .notdef"
        );
    }

    // Install the stack: resolve-by-coverage + itemize + per-item shaping.
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

#[doc = include_str!("windowed_counter.doc/font_handle.md")]
fn font_handle(id: obscur_kit::text::font::FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("windowed_counter.doc/counter_stylesheet.md")]
fn counter_stylesheet() -> Stylesheet {
    // The `:hover` rules are what prove the domain input path drives styling:
    // they only take effect because `handle_pointer` records HOVER on the
    // hovered node's pseudo-state, which the runtime's matcher then honours.
    let css = "\
.counter         { background-color: #1e2636; width: 100%; height: 100%; }\n\
.bar             { width: 100%; }\n\
.label           { background-color: #2b3348; height: 50%; }\n\
.decrement       { background-color: #d9534f; height: 25%; }\n\
.increment       { background-color: #5cb85c; height: 25%; }\n\
.decrement:hover { background-color: #ef7b77; }\n\
.increment:hover { background-color: #7fd39b; }\n\
";
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("windowed_counter.doc/tagged.md")]
fn tagged(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("windowed_counter.doc/build_counter_view.md")]
fn build_counter_view(count: i32) -> View {
    // Mixed-script label (Latin + CJK): on a machine with both a Latin primary
    // and a CJK fallback registered, each script renders from its covering font
    // via the coverage resolver; with only the primary, the CJK part is .notdef.
    let label = tagged(
        "bar label",
        vec![Text::new(format!("Count 数字: {count}")).into_view_node()],
    );
    let minus = button_bar("bar decrement", "-", CMD_DECREMENT);
    let plus = button_bar("bar increment", "+", CMD_INCREMENT);
    let container = Container::new()
        .child(label)
        .child(minus)
        .child(plus)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())));
    View::single(container)
}

#[doc = include_str!("windowed_counter.doc/button_bar.md")]
fn button_bar(class: &str, label: &str, command: InternedSymbol) -> ViewNode {
    Button::new(label, command)
        .view()
        .into_fragment()
        .nodes()
        .to_vec()
        .remove(0)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

#[doc = include_str!("windowed_counter.doc/main.md")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit windowed counter ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)\n");

    // A `UserEvent` loop, because the backend's handler is an
    // `ApplicationHandler<UserEvent>`: that is the §3.5 cross-thread wake-up
    // channel a frame scheduler nudges the loop through.
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = WindowedCounter::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}

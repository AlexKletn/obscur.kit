//! The winit [`ApplicationHandler`] behind [`App::run`](super::App::run): it owns
//! the window, the GPU-backed Vello renderer, the swapchain surface, and the
//! embeddable [`UiRuntime`], and composes the kit's own facilities
//! ([`KitBootstrap`], [`CssPipeline`], the winit converters) rather than
//! hand-wiring them again. Lifted from the `uikit_window` / `windowed_counter`
//! examples so an application does not spell out ~700 lines to open one window.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey as WinitPhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

use super::AppError;
use super::app::FrameInfo;
use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::css::source::CssSource;
use crate::geometry::{Point, Size};
use crate::identity::{InternedSymbol, RuntimeId};
use crate::interaction::command::KeyEvent;
use crate::interaction::pointer::{PointerButtons, PointerEvent, PointerId};
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::text::ime::{CompositionEvent, Preedit};
use crate::view::view::{View, ViewNode};
use crate::window_winit::conversion::{KeyboardConverter, PointerConverter};

/// Pixels scrolled per wheel line, when the platform reports whole lines rather
/// than raw pixels.
const WHEEL_LINE_STEP: f32 = 48.0;

/// Rolling per-phase frame timings, printed every [`REPORT_EVERY`] presented
/// frames when `OBSCUR_FRAME_STATS=1` is set.
///
/// This exists because the kit measures nothing itself: §7.9's
/// [`PerformanceSpan`](crate::diagnostics::performance::PerformanceSpan) carries a
/// name and a subsystem but **no duration**, and the DevTools protocol has no
/// timing field, so neither telemetry nor an attached inspector can answer "why is
/// this slow". Until a span records elapsed time, this is the honest way to profile
/// a running app.
#[derive(Default)]
struct FrameStats {
    frames: u32,
    build: std::time::Duration,
    snapshot: std::time::Duration,
    present: std::time::Duration,
    worst: std::time::Duration,
}

/// How many presented frames each stats line covers.
const REPORT_EVERY: u32 = 60;

impl FrameStats {
    /// Folds one presented frame in, printing a summary every [`REPORT_EVERY`].
    fn record(
        &mut self,
        build: std::time::Duration,
        snapshot: std::time::Duration,
        present: std::time::Duration,
    ) {
        self.frames += 1;
        self.build += build;
        self.snapshot += snapshot;
        self.present += present;
        self.worst = self.worst.max(build + snapshot + present);
        if self.frames < REPORT_EVERY {
            return;
        }
        let n = f64::from(self.frames);
        let ms = |total: std::time::Duration| total.as_secs_f64() * 1000.0 / n;
        let total = ms(self.build) + ms(self.snapshot) + ms(self.present);
        println!(
            "[frame-stats] {n:.0} frames | build {:.2}ms | snapshot {:.2}ms | present {:.2}ms | \
             total {total:.2}ms (~{:.0} fps if saturated) | worst {:.2}ms",
            ms(self.build),
            ms(self.snapshot),
            ms(self.present),
            1000.0 / total.max(0.001),
            self.worst.as_secs_f64() * 1000.0,
        );
        *self = Self::default();
    }
}

/// Builds the runtime, forwards the app's root builder + command handlers + CSS,
/// then opens the window and runs the event loop to completion.
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    builder: Box<dyn FnMut() -> View + 'static>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
    virtual_clicks: Vec<(InternedSymbol, Box<dyn FnMut(usize, u32) + 'static>)>,
) -> Result<(), AppError> {
    run_inner(
        title,
        app_id,
        size,
        stylesheets,
        builder,
        commands,
        scroll,
        None,
        virtual_lists,
        virtual_clicks,
    )
}

/// [`run`] for a builder that reads the frame's [`FrameInfo`].
///
/// The runner owns the window, so it owns the only authoritative viewport: it keeps
/// the current one in a cell, refreshes it whenever the surface changes, and hands
/// the builder a snapshot each frame. That is what lets an app window its own list
/// to what is visible instead of building a node per item.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_with_frame(
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    mut builder: Box<dyn FnMut(&FrameInfo) -> View + 'static>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
    virtual_clicks: Vec<(InternedSymbol, Box<dyn FnMut(usize, u32) + 'static>)>,
) -> Result<(), AppError> {
    let info = Rc::new(Cell::new(FrameInfo {
        viewport: Size::new(size.0, size.1),
        scale: 1.0,
    }));
    let seen = Rc::clone(&info);
    let wrapped: Box<dyn FnMut() -> View + 'static> = Box::new(move || builder(&seen.get()));
    run_inner(
        title,
        app_id,
        size,
        stylesheets,
        wrapped,
        commands,
        scroll,
        Some(info),
        virtual_lists,
        virtual_clicks,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_inner(
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    builder: Box<dyn FnMut() -> View + 'static>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    frame_info: Option<Rc<Cell<FrameInfo>>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
    virtual_clicks: Vec<(InternedSymbol, Box<dyn FnMut(usize, u32) + 'static>)>,
) -> Result<(), AppError> {
    let (width, height) = size;
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));

    // The root view builder and command handlers are the app's own; forward them
    // into the runtime verbatim (the same shapes `set_root_view` / `on_command`
    // take). A `Box<dyn FnMut() -> View>` is itself `FnMut() -> View`.
    runtime.set_root_view(builder);
    for (command, handler) in commands {
        runtime.on_command(command, handler);
    }
    for (id, build) in virtual_lists {
        runtime.register_virtual_list(id, build);
    }
    for (id, handler) in virtual_clicks {
        runtime.on_virtual_row_click(id, handler);
    }

    let boot = KitBootstrap::new();

    // Defaults-on that need no window (§5.18 / §3.3): the UA default sheet (below
    // every author sheet), the accumulated author CSS through the motion-aware
    // pipeline, filesystem images, and the threaded blocking executor (correct for
    // a standalone window — it is not a headless host).
    boot.install_default_stylesheet(&mut runtime);
    let mut css = CssPipeline::new();
    let sources: Vec<CssSource> = stylesheets.into_iter().map(CssSource::anonymous).collect();
    css.install(&mut runtime, &sources);
    install_resources(&boot, &mut runtime);

    let mut app = AppRunner {
        title,
        app_id,
        size,
        runtime,
        _css: css,
        window: None,
        surface: None,
        device: None,
        renderer: None,
        scale_factor: 1.0,
        cursor: (0.0, 0.0),
        epoch: std::time::Instant::now(),
        init_failure: None,
        scroll,
        frame_info,
        stats: std::env::var_os("OBSCUR_FRAME_STATS").map(|_| FrameStats::default()),
    };

    let event_loop = EventLoop::new().map_err(|error| AppError::EventLoop(error.to_string()))?;
    event_loop
        .run_app(&mut app)
        .map_err(|error| AppError::EventLoop(error.to_string()))?;

    // A GPU/window failure exits the loop cleanly on `resumed`, recording the
    // reason so `run` can report it rather than returning `Ok` on a blank window.
    match app.init_failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Installs the filesystem image resources and the threaded blocking executor when
/// their adapters are compiled in (both are in the `full` set the windowed feature
/// gate implies). A standalone window is not a headless host, so the *real*
/// executor is correct here — a `url()` image lands a frame later and
/// [`poll_resource_tasks`](UiRuntime::poll_resource_tasks) on wake repaints for it.
#[allow(unused_variables)]
fn install_resources(boot: &KitBootstrap, runtime: &mut UiRuntime) {
    #[cfg(all(feature = "resource-fs", feature = "resource-decode"))]
    {
        boot.install_image_resources(runtime, ".");
        boot.install_vector_resources(runtime);
    }
    #[cfg(feature = "task-smol")]
    {
        let _spawner = boot.install_blocking_executor(runtime);
    }
}

/// Installs the real text stack over the host's system fonts (bundled last-resort
/// fallback appended by [`install_text_stack`](KitBootstrap::install_text_stack)),
/// and hands the renderer the *same* id→bytes assignment so scene font handles
/// line up with what it rasterizes — the codified fix for the id-matching problem.
#[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
fn install_text(boot: &KitBootstrap, runtime: &mut UiRuntime, renderer: &mut VelloRenderer) {
    #[cfg(feature = "font-fontconfig")]
    let font_set = boot.system_font_set().unwrap_or_default();
    #[cfg(not(feature = "font-fontconfig"))]
    let font_set = crate::bootstrap::FontSet::new();

    let installed = boot.install_text_stack(runtime, &font_set);
    for (handle, bytes) in installed.iter() {
        renderer.register_font(handle, bytes.to_vec());
    }
}

#[cfg(not(all(feature = "text-harfrust", feature = "text-unicode")))]
fn install_text(_boot: &KitBootstrap, _runtime: &mut UiRuntime, _renderer: &mut VelloRenderer) {}

/// The winit application the runner drives: the window, the GPU renderer, the
/// swapchain surface, and the embeddable runtime, all created lazily on `resumed`.
struct AppRunner {
    title: String,
    app_id: String,
    size: (f32, f32),
    runtime: UiRuntime,
    /// The CSS pipeline is kept alive for the app's lifetime (it owns the §4.12
    /// parse cache); it is installed once in [`run`] and retained here.
    _css: CssPipeline,
    window: Option<Arc<Window>>,
    surface: Option<VelloSurface>,
    device: Option<wgpu::Device>,
    renderer: Option<VelloRenderer>,
    /// The display's HiDPI device-pixel ratio (winit `scale_factor`).
    scale_factor: f64,
    /// Last cursor position in physical px, so a `MouseInput` (which carries none)
    /// dispatches at the cursor.
    cursor: (f64, f64),
    /// The instant the app started, and the origin of every pointer event's
    /// logical timestamp — see [`now_micros`](AppRunner::now_micros).
    epoch: std::time::Instant,
    /// A window/GPU failure recorded on `resumed` (which cannot return a `Result`)
    /// so [`run`] can surface it instead of returning `Ok` on a blank window.
    init_failure: Option<AppError>,
    /// The app's leftover-wheel handler, if it registered one.
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    /// The viewport snapshot a frame-aware builder reads, refreshed on every
    /// surface change.
    frame_info: Option<Rc<Cell<FrameInfo>>>,
    /// Per-phase frame timings, present only when `OBSCUR_FRAME_STATS=1` is set.
    stats: Option<FrameStats>,
}

impl AppRunner {
    /// Microseconds since the app started — the logical timestamp every pointer
    /// event this runner dispatches carries.
    ///
    /// The interaction domain never reads a clock (it takes the time as data), so
    /// supplying one is the platform adapter's job, and nothing did it: every
    /// event went out stamped `0`. That is not a cosmetic gap — the runtime's
    /// multi-click test is *same target, within the click window, within the
    /// radius*, and with every press at time `0` the window condition
    /// (`0 - 0 <= 500ms`) is vacuously true. A press on the same spot was a
    /// double click no matter how many seconds apart, so a file-manager row
    /// clicked twice a minute apart opened the directory. Only the elapsed clock
    /// makes the window mean anything.
    #[allow(clippy::cast_possible_truncation)]
    fn now_micros(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64
    }

    /// The device-pixel scale the runtime renders at: the display density, floored
    /// so a degenerate `0` never divides the viewport.
    #[allow(clippy::cast_possible_truncation)]
    fn scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    /// Points the runtime's viewport + device scale at the swapchain's physical
    /// extent and the display density — the kit's HiDPI mapping in one call.
    #[allow(clippy::cast_possible_truncation)]
    fn sync_surface(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            self.runtime.set_surface(
                Size::new(config.width as f32, config.height as f32),
                self.scale(),
            );
            if let Some(info) = self.frame_info.as_ref() {
                let scale = self.scale();
                info.set(FrameInfo {
                    // Logical, because that is the space the app's CSS and its
                    // layout are in; the physical extent is the runner's business.
                    viewport: Size::new(config.width as f32 / scale, config.height as f32 / scale),
                    scale,
                });
            }
        }
    }

    /// Runs a scheduler-gated reactive frame, reads the committed scene, and
    /// presents it on-screen via the Vello offscreen-render + blit path. A lost
    /// swapchain is reconfigured and the frame skipped (the next redraw recovers).
    #[allow(clippy::cast_precision_loss)]
    fn render_frame(&mut self) {
        // Only present when a frame was actually produced. A redraw request that
        // finds nothing due — a bare cursor move over unchanged UI, which winit
        // delivers continuously — would otherwise rebuild the view, re-snapshot and
        // re-present an identical scene on every pointer twitch. The runtime is the
        // authority on whether anything changed: it schedules a frame itself when
        // hover, focus, animation or a landed resource makes one necessary.
        let started = std::time::Instant::now();
        if self.runtime.run_due_frame_reactive().is_none() {
            return;
        }
        let built = started.elapsed();
        let snapshot_at = std::time::Instant::now();
        let snapshot = self.runtime.snapshot();
        let snapshotted = snapshot_at.elapsed();
        let present_at = std::time::Instant::now();

        let (Some(surface), Some(renderer)) = (self.surface.as_ref(), self.renderer.as_mut())
        else {
            return;
        };

        let physical = Size::new(
            surface.config().width as f32,
            surface.config().height as f32,
        );
        let submission = FrameSubmission::new(RenderFrame::new(snapshot, physical));
        if let Err(error) = renderer.present_to_surface(&submission, surface) {
            eprintln!("present failed: {error:?}");
            if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref()) {
                surface.configure(device);
            }
        }
        if let Some(stats) = self.stats.as_mut() {
            stats.record(built, snapshotted, present_at.elapsed());
        }
    }

    /// Requests a redraw of the live window, if any.
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor, reusing
    /// [`PointerConverter::kind`] to map the winit state (the converter has no
    /// public builder for a button transition *at a position*).
    #[allow(clippy::cast_possible_truncation)]
    /// Builds the press/release for a left-button change.
    ///
    /// It goes through [`PointerConverter`]'s `pressed`/`released` builders rather
    /// than `PointerEvent::new`, because those are the ones that mark the event
    /// **trusted** (§2.3). This is the platform-adapter path — a real user gesture —
    /// and a gated default action (the click → command dispatch) only runs for a
    /// trusted event, so building the event by hand here silently disabled every
    /// click in every `App`.
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        let at = PhysicalPosition::new(self.cursor.0, self.cursor.1);
        let converter = PointerConverter::new();
        match state {
            ElementState::Pressed => converter.pressed(
                PointerId::MOUSE,
                at,
                PointerButtons::from_bits(PointerButtons::PRIMARY),
            ),
            ElementState::Released => {
                converter.released(PointerId::MOUSE, at, PointerButtons::new())
            }
        }
        .with_timestamp(self.now_micros())
    }
}

impl ApplicationHandler for AppRunner {
    /// Creates the window + GPU renderer once the platform is ready, wires the text
    /// stack, enables IME, and schedules the first frame. A window or GPU failure
    /// records the reason and exits the loop (never panics — the headless CI path).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(unused_mut))]
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_transparent(true)
            .with_inner_size(LogicalSize::new(
                f64::from(self.size.0),
                f64::from(self.size.1),
            ));
        // The Wayland `app_id` / X11 `WM_CLASS` — the stable identity a compositor
        // matches window rules on (Hyprland's `class`, sway's `app_id`) and the one
        // a taskbar keys off. winit exposes it only through the per-backend
        // extension traits, and setting both is correct: they write different
        // platform fields, so an X11 session and a Wayland one each get the id.
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            {
                use winit::platform::wayland::WindowAttributesExtWayland as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
            {
                use winit::platform::x11::WindowAttributesExtX11 as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
        }
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.init_failure = Some(AppError::Window(error.to_string()));
                event_loop.exit();
                return;
            }
        };
        // Allow the platform IME to deliver composition events (spec §5.12), so
        // text input works out of the box; the `WindowEvent::Ime` arm drives it.
        window.set_ime_allowed(true);

        let (surface, config, device, queue) = match init_gpu(&window) {
            Ok(gpu) => gpu,
            Err(error) => {
                self.init_failure = Some(AppError::Gpu(error));
                event_loop.exit();
                return;
            }
        };
        let vello_surface = VelloSurface::new(surface, config);
        vello_surface.configure(&device);
        let vello_device = match VelloDevice::new(device.clone(), queue) {
            Ok(vello_device) => vello_device,
            Err(error) => {
                self.init_failure = Some(AppError::Gpu(error.to_string()));
                event_loop.exit();
                return;
            }
        };
        let mut renderer = VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
        install_text(&KitBootstrap::new(), &mut self.runtime, &mut renderer);

        self.scale_factor = window.scale_factor();
        self.surface = Some(vello_surface);
        self.device = Some(device);
        self.renderer = Some(renderer);
        self.window = Some(window);
        self.sync_surface();
        self.runtime.request_frame(FrameReason::UiCommit);
        self.request_redraw();
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
                self.sync_surface();
                self.runtime.request_frame(FrameReason::WindowChange);
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.sync_surface();
                self.runtime.request_frame(FrameReason::WindowChange);
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new()
                    .moved(PointerId::MOUSE, position)
                    .with_timestamp(self.now_micros());
                self.runtime.handle_pointer(moved);
                self.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                let event = self.pointer_button_event(state);
                self.runtime.handle_pointer(event);
                self.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // A wheel notch scrolls the `overflow: scroll` container under the
                // cursor. winit reports whole lines or raw pixels; normalize both to
                // a pixel step, and invert so that wheel-down (negative lines)
                // increases scroll_top — i.e. reveals content further down. The
                // cursor is passed in the same (physical) space the pointer
                // converter uses, so the hit that finds the container matches the
                // one a click would make.
                let step = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => -lines * WHEEL_LINE_STEP,
                    #[allow(clippy::cast_possible_truncation)]
                    MouseScrollDelta::PixelDelta(position) => -(position.y as f32),
                };
                #[allow(clippy::cast_possible_truncation)]
                let at = Point::new(self.cursor.0 as f32, self.cursor.1 as f32);
                if self.runtime.handle_scroll(step, at).is_some() {
                    self.runtime.request_frame(FrameReason::WindowChange);
                    self.request_redraw();
                } else if let Some(handler) = self.scroll.as_mut() {
                    // No `overflow: scroll` container took it, so the app may want
                    // it: an app that windows its own list owns its scroll position.
                    handler(step / self.scale_factor.max(0.05) as f32);
                    self.runtime.request_frame(FrameReason::UiCommit);
                    self.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        self.runtime
                            .handle_key(KeyEvent::new(physical, logical, pressed));
                        self.request_redraw();
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
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    /// Polls for landed resource loads each turn: the real blocking executor lands
    /// a `url()` image a frame later, so an otherwise idle UI repaints when it
    /// arrives (spec §1.5 / §3.3).
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.runtime.poll_resource_tasks() > 0 {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
    }
}

/// Initialises a wgpu swapchain surface + device/queue for on-screen rendering
/// from an `Arc<Window>` (the standard pollster `Instance → Surface → Adapter →
/// Device → Queue` chain, exactly as the windowed examples do).
fn init_gpu(
    window: &Arc<Window>,
) -> Result<
    (
        wgpu::Surface<'static>,
        wgpu::SurfaceConfiguration,
        wgpu::Device,
        wgpu::Queue,
    ),
    String,
> {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(window.clone())
        .map_err(|error| error.to_string())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|error| error.to_string())?;
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or_else(|| "surface unsupported by adapter".to_owned())?;
    // Let the root's own `background-color` decide whether the window is see-through,
    // with no API to opt into: `get_default_config` picks `Opaque`, which throws the
    // alpha away at composite time, so an `rgba(…, 0.7)` root would paint correctly
    // and still show up solid. Asking for a premultiplied surface hands the alpha to
    // the compositor instead — Vello already clears to `TRANSPARENT`, so an opaque
    // root still fills every pixel with alpha 1 and looks exactly as before. This is
    // therefore not a mode: it is the absence of one, and the only thing that decides
    // the result is the stylesheet. Where the compositor offers neither premultiplied
    // nor postmultiplied alpha the default stands and the window stays opaque.
    let alpha_modes = surface.get_capabilities(&adapter).alpha_modes;
    for mode in [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
    ] {
        if alpha_modes.contains(&mode) {
            config.alpha_mode = mode;
            break;
        }
    }
    // `get_default_config` picks `Fifo` — vsync — which is the right default: it
    // paces to the display and burns no power on frames nobody sees. But it also
    // makes `present` block until the next refresh, so a profiling run cannot tell
    // real GPU cost from waiting, and a frame that misses the deadline reads the
    // same as one that is merely paced. `OBSCUR_NO_VSYNC=1` unpaces the swapchain
    // so a measurement shows the true frame cost.
    if std::env::var_os("OBSCUR_NO_VSYNC").is_some() {
        let supported = surface.get_capabilities(&adapter).present_modes;
        for mode in [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox] {
            if supported.contains(&mode) {
                config.present_mode = mode;
                break;
            }
        }
        println!("[present-mode] {:?}", config.present_mode);
    }
    Ok((surface, config, device, queue))
}

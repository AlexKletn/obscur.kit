//! The winit [`ApplicationHandler`] behind [`App::run`](super::App::run): it owns
//! the window, the GPU-backed Vello renderer, the swapchain surface, and the
//! embeddable [`UiRuntime`], and composes the kit's own facilities
//! ([`KitBootstrap`], [`CssPipeline`], the winit converters) rather than
//! hand-wiring them again. Lifted from the `uikit_window` / `windowed_counter`
//! examples so an application does not spell out ~700 lines to open one window.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey as WinitPhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

use super::AppError;
use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::css::source::CssSource;
use crate::geometry::{Point, Size};
use crate::identity::{InternedSymbol, RuntimeId};
use crate::interaction::command::KeyEvent;
use crate::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::text::ime::{CompositionEvent, Preedit};
use crate::view::view::View;
use crate::window_winit::conversion::{KeyboardConverter, PointerConverter};

/// Pixels scrolled per wheel line, when the platform reports whole lines rather
/// than raw pixels.
const WHEEL_LINE_STEP: f32 = 48.0;

/// Builds the runtime, forwards the app's root builder + command handlers + CSS,
/// then opens the window and runs the event loop to completion.
pub(super) fn run(
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    builder: Box<dyn FnMut() -> View + 'static>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
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
        init_failure: None,
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
    /// A window/GPU failure recorded on `resumed` (which cannot return a `Result`)
    /// so [`run`] can surface it instead of returning `Ok` on a blank window.
    init_failure: Option<AppError>,
}

impl AppRunner {
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
        }
    }

    /// Runs a scheduler-gated reactive frame, reads the committed scene, and
    /// presents it on-screen via the Vello offscreen-render + blit path. A lost
    /// swapchain is reconfigured and the frame skipped (the next redraw recovers).
    #[allow(clippy::cast_precision_loss)]
    fn render_frame(&mut self) {
        self.runtime.run_due_frame_reactive();
        let snapshot = self.runtime.snapshot();

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
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
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
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
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
    let config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or_else(|| "surface unsupported by adapter".to_owned())?;
    Ok((surface, config, device, queue))
}

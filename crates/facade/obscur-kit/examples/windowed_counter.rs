//! End-to-end **windowed** capstone for the `obscur-kit` facade.
//!
//! Unlike `counter_headless.rs` (no display, no GPU), this example wires the
//! *real* platform backends the `full` feature set unlocks — winit 0.30 for the
//! window and event loop, and Vello/wgpu 29 for the GPU renderer — and drives
//! them through the toolkit's public umbrella crate only (`obscur_kit::*`). It
//! opens a genuine OS window and **presents the painted counter on-screen** each
//! frame, running until the window is closed.
//!
//! Run it on a real machine (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example windowed_counter --features full`
//!
//! # Wiring
//!
//! 1. **Window / event loop** — a winit 0.30 [`ApplicationHandler`]: `resumed`
//!    creates the window (held as an [`Arc`] so wgpu can build a
//!    `Surface<'static>` from it) and initialises wgpu + the swapchain surface;
//!    `window_event` handles resize, close, and redraw. (The toolkit also ships
//!    `obscur_kit::window_winit::backend::WinitWindowBackend`, the pull-based
//!    [`WindowBackend`] port; here we create the window directly in `resumed`,
//!    which is the simplest correct path and keeps the render-side wiring the
//!    focus of the example.)
//! 2. **GPU device + surface** — wgpu `Instance → Surface → Adapter → Device →
//!    Queue`, with the device/queue handed to `VelloDevice::new` so the adapter
//!    never owns the async `request_adapter` path (spec §2.7). The surface is
//!    wrapped in a `VelloSurface` that owns the swapchain lifecycle.
//! 3. **Frame driver** — the deterministic [`HeadlessHost`] runs the real
//!    pipeline (reconcile → style → layout → scene) and exposes the committed
//!    scene as a `SceneSnapshot`. There is no separate "windowed" host; reusing
//!    the headless host as the frame driver is intended.
//! 4. **Present** — the snapshot is packaged as a `RenderFrame` +
//!    `FrameSubmission` and submitted to `VelloRenderer::present_to_surface`,
//!    which rasterizes the scene into an offscreen Vello texture and then blits
//!    that texture onto the acquired swapchain image (Vello 0.9 has no
//!    render-to-surface helper; offscreen-render + blit is the documented
//!    pattern). The counter is therefore genuinely visible on-screen.
//!
//! `render_styled` matches the compiled `.counter` stylesheet against the
//! reconciled tree, so the committed display list carries a real fill — that
//! fill is what is rasterized and presented.
//!
//! # Environment note
//!
//! This example cannot run in the headless dev/CI environment (no display, no
//! GPU adapter), so it is only ever *compiled* there. On a real machine it opens
//! a window and continuously presents the painted counter until closed.
#![forbid(unsafe_code)]

use std::sync::Arc;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::headless::HeadlessHost;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Commands the two buttons fire when activated (interned symbol handles).
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

/// The winit application: it owns the window, the GPU-backed renderer, the
/// swapchain surface, the deterministic frame driver, and the counter UI state.
struct WindowedCounter {
    /// The live winit window (an [`Arc`] so wgpu can build a `Surface<'static>`
    /// from it), created lazily in [`ApplicationHandler::resumed`].
    window: Option<Arc<Window>>,
    /// The on-screen swapchain surface, created alongside the window.
    surface: Option<VelloSurface>,
    /// The wgpu device, cached so resizes can reconfigure the surface without
    /// reaching into the renderer (a `wgpu::Device` is a cheap Arc handle).
    device: Option<wgpu::Device>,
    /// The Vello/wgpu renderer, attached once the GPU device is created.
    renderer: Option<VelloRenderer>,
    /// The deterministic host that runs the real pipeline each frame.
    host: HeadlessHost,
    /// The compiled stylesheet matched against the tree each frame so the
    /// `.counter` container paints a real fill.
    style_sheet: Stylesheet,
    /// The "-" button (fixed; drives the decrement command symbol).
    decrement: Button,
    /// The "+" button whose headless state machine we drive each frame.
    increment: Button,
    /// The current counter value shown by the label.
    count: i64,
}

impl WindowedCounter {
    /// Builds the application with a booted [`HeadlessHost`] and the two-button
    /// counter UI, but no window, surface, or GPU device yet (those are created
    /// on `resumed`).
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            host: KitBootstrap::new().with_runtime_id(1).headless_host(),
            style_sheet: counter_stylesheet(),
            decrement: Button::new("-", CMD_DECREMENT),
            increment: Button::new("+", CMD_INCREMENT),
            count: 0,
        }
    }

    /// Renders one frame: advance UI state, build the view, drive the pipeline,
    /// package the scene, and present it on-screen via the Vello surface path.
    fn render_frame(&mut self) {
        // Nothing to draw until the window, surface, and renderer all exist.
        let (Some(window), Some(surface), Some(renderer)) =
            (self.window.as_ref(), self.surface.as_ref(), self.renderer.as_mut())
        else {
            return;
        };

        // --- Stage 1: advance UI state via the button's headless machine. -----
        // A full press + release on an enabled button reports activation.
        self.increment.press();
        if self.increment.release() {
            self.count += 1;
        }

        // --- Stage 2: build the counter view from the components tier. --------
        let view = build_counter_view(self.count, &self.decrement, &self.increment);

        // --- Stage 3: drive the real pipeline and read the committed scene. ---
        // `render_styled` matches the stylesheet against the reconciled tree, so
        // the `.counter` rule paints the container.
        let frame = self
            .host
            .render_styled(&view, std::slice::from_ref(&self.style_sheet));
        let snapshot = self.host.snapshot();

        // --- Stage 4: package the snapshot into a submission for the port. ----
        // The physical viewport is the surface's configured swapchain extent.
        // u32 -> f32 for window extents is intentional and lossless in practice
        // (cast_precision_loss is allowed workspace-wide).
        let physical_size = Size::new(surface.config().width as f32, surface.config().height as f32);
        let render_frame = RenderFrame::new(snapshot, physical_size);
        let submission = FrameSubmission::new(render_frame);

        // --- Stage 5: present on-screen (offscreen render + blit-to-surface). -
        match renderer.present_to_surface(&submission, surface) {
            Ok(result) => println!(
                "frame {}: pipeline id={} composed={} | vello -> {result:?} (presented)",
                renderer.frames_rendered(),
                frame.id().raw(),
                frame.composed(),
            ),
            Err(error) => {
                eprintln!("present failed: {error:?}");
                // A lost/outdated swapchain is recoverable: reconfigure and skip
                // this frame. The next redraw acquires a fresh texture.
                if let (Some(surface), Some(device)) =
                    (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }

        // Keep the counter animating: ask for another frame so the window is a
        // genuine, continuously-redrawing surface until the user closes it.
        window.request_redraw();
    }
}

impl ApplicationHandler for WindowedCounter {
    /// Creates the window, the swapchain surface, and the GPU-backed renderer
    /// once the platform is ready.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `resumed` can fire more than once (e.g. on Android); build only once.
        if self.window.is_some() {
            return;
        }

        // 1. Create the real winit window, owned via `Arc` so wgpu can build a
        //    `Surface<'static>` from a cloned handle.
        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — windowed counter")
            .with_inner_size(LogicalSize::new(480.0, 240.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        // 2. Initialise wgpu + the swapchain surface and build the Vello
        //    renderer on top of the host-provided device/queue.
        match init_gpu(&window) {
            Ok((surface, config, device, queue)) => {
                let vello_surface = VelloSurface::new(surface, config);
                vello_surface.configure(&device);
                match VelloDevice::new(device.clone(), queue) {
                    Ok(vello_device) => {
                        let renderer =
                            VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        // Kick off the first frame.
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

    /// Routes window events: resize reconfigures the swapchain, redraw presents a
    /// frame, and close exits.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device)) =
                    (self.surface.as_mut(), self.device.as_ref())
                {
                    surface.resize(device, new_size.width, new_size.height);
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

/// Initialises a wgpu swapchain surface + device/queue for **on-screen**
/// rendering, from an `Arc<Window>`.
///
/// The window handle is cloned into `Instance::create_surface`, yielding a
/// `Surface<'static>`; the adapter is requested compatible with that surface,
/// and the surface's default configuration is derived from the current window
/// size. wgpu's async `request_adapter` / `request_device` are driven to
/// completion with `pollster::block_on`.
///
/// # Errors
///
/// Returns an error if the surface cannot be created, no compatible GPU adapter
/// is available (as in a headless environment), the device cannot be created, or
/// the surface exposes no supported default configuration.
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

/// The authored stylesheet: one rule per class, each painting a background +
/// size. Layout is flow (vertical stacking) — `display` is not yet mapped and
/// the property set has no margin/align — so the bars stack top-to-bottom
/// inside the dark `.counter` panel.
fn counter_stylesheet() -> Stylesheet {
    let css = "\
.counter   { background-color: #1e2636; width: 100%; height: 100%; }\n\
.bar       { width: 100%; }\n\
.label     { background-color: #3a4560; height: 50%; }\n\
.decrement { background-color: #d9534f; height: 25%; }\n\
.increment { background-color: #5cb85c; height: 25%; }\n\
";
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Wraps `children` in a container element tagged with the space-separated
/// `class` string, so an authored rule (e.g. `.btn`) matches and paints it.
fn tagged(class: &str, children: Vec<ViewNode>) -> ViewNode {
    Container::new()
        .with_children(children)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// Builds the counter UI: a dark `.counter` panel stacking a label bar and the
/// two buttons, each wrapped so it carries a class the stylesheet paints.
fn build_counter_view(count: i64, decrement: &Button, increment: &Button) -> View {
    let label = tagged("bar label", vec![Text::new(format!("Count: {count}")).into_view_node()]);
    let minus = tagged("bar decrement", button_nodes(decrement));
    let plus = tagged("bar increment", button_nodes(increment));
    let container = Container::new()
        .child(label)
        .child(minus)
        .child(plus)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())));
    View::single(container)
}

/// Extracts a button's desired [`ViewNode`]s from the [`View`] it renders, so
/// they can be composed as children of another element.
fn button_nodes(button: &Button) -> Vec<ViewNode> {
    button.view().into_fragment().nodes().to_vec()
}

/// Builds the winit event loop and runs the application to completion.
///
/// # Errors
///
/// Propagates any event-loop construction or run error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit windowed counter ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)\n");

    let event_loop = EventLoop::new()?;
    let mut app = WindowedCounter::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

//! End-to-end **windowed** capstone for the `obscur-kit` facade.
//!
//! Unlike `counter_headless.rs` (no display, no GPU), this example wires the
//! *real* platform backends the `full` feature set unlocks — winit 0.30 for the
//! window and event loop, and Vello/wgpu 29 for the GPU renderer — and drives
//! them through the toolkit's public umbrella crate only (`obscur_kit::*`). It
//! opens a genuine OS window and **presents the painted counter on-screen**,
//! redrawing on input and resize, running until the window is closed. Moving the
//! pointer over a bar lights its `:hover` style through the real input path, and
//! clicking the lower red bar decrements the counter while the green bar
//! increments it.
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
//! 3. **Frame driver** — the embeddable [`UiRuntime`] runs the real,
//!    *scheduler-gated* pipeline (reconcile → pseudo-aware match → style → layout
//!    → scene) and exposes the committed scene as a `SceneSnapshot`. Input,
//!    resize, and the counter's value each `request_frame` a reason;
//!    [`UiRuntime::run_due_frame`] then produces at most one frame per redraw
//!    when something is actually due, and the runtime retains the last frame's
//!    layout boxes so its hit-test index can map a pointer back to a node.
//! 4. **Present** — the snapshot is packaged as a `RenderFrame` +
//!    `FrameSubmission` and submitted to `VelloRenderer::present_to_surface`,
//!    which rasterizes the scene into an offscreen Vello texture and then blits
//!    that texture onto the acquired swapchain image (Vello 0.9 has no
//!    render-to-surface helper; offscreen-render + blit is the documented
//!    pattern). The counter is therefore genuinely visible on-screen.
//!
//! The runtime's pseudo-aware matcher matches the compiled `.counter` /
//! `.bar` / `:hover` stylesheet against the reconciled tree, so the committed
//! display list carries a real fill — that fill is what is rasterized and
//! presented.
//!
//! # Input
//!
//! Pointer input flows through the **real domain path**, not a geometric hack:
//! winit → [`PointerConverter`] → [`UiRuntime::handle_pointer`] → hit-test →
//! hover/focus → `:hover`/`:focus` restyle. Concretely:
//!
//! - `CursorMoved` is converted by [`PointerConverter::moved`] into a domain
//!   [`Move`](obscur_kit::interaction::pointer::PointerEventKind::Move) event and
//!   dispatched; the runtime hit-tests it against the retained layout boxes,
//!   updates the hover chain, and (on a change) schedules an `Input` frame — so
//!   hovering a bar visibly swaps to its lighter `:hover` background.
//! - `MouseInput` (left press) builds a
//!   [`Down`](obscur_kit::interaction::pointer::PointerEventKind::Down) event —
//!   assembled directly with [`PointerConverter::kind`] mapping the winit state,
//!   since the converter has no public builder for a button transition *at a
//!   position* — and dispatches it to drive focus.
//!
//! The counter *value* is still changed geometrically (the node→"which button"
//! mapping needs component instances this example does not carry), so a left
//! press on the lower `.decrement`/`.increment` region also nudges `count`; the
//! hover/focus **visuals** are what the real input path now drives.
//!
//! # Text
//!
//! The counter's labels ("Count: N", "-", "+") are real text. Text is shaped
//! once in the `obscur-text` domain, so each `DrawText` command already carries
//! positioned glyphs. Fonts are a host resource (like the GPU device): `resumed`
//! loads a system font off disk and hands its bytes to
//! [`VelloRenderer::set_font`]; the adapter resolves the run's glyphs against
//! that font and Vello rasterizes them into the same offscreen texture the blit
//! path presents. With no font found, the shapes still paint and only the glyphs
//! are skipped.
//!
//! # Environment note
//!
//! This example cannot run in the headless dev/CI environment (no display, no
//! GPU adapter), so it is only ever *compiled* there. On a real machine it opens
//! a window, presents the painted counter, and responds to clicks until closed.
#![forbid(unsafe_code)]

use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use obscur_kit::window_winit::conversion::PointerConverter;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Commands the two buttons fire when activated (interned symbol handles).
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

/// The winit application: it owns the window, the GPU-backed renderer, the
/// swapchain surface, the embeddable [`UiRuntime`] frame driver, and the counter
/// UI state.
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
    /// The embeddable runtime that owns the retained tree, scene, scheduler, and
    /// live interaction (hover/focus) state, and runs the real pipeline when a
    /// frame is due. Pointer input is dispatched into it via `handle_pointer`.
    runtime: UiRuntime,
    /// The compiled stylesheet matched against the tree each frame so the
    /// `.counter` container paints a real fill.
    style_sheet: Stylesheet,
    /// The "-" button (fixed; drives the decrement command symbol).
    decrement: Button,
    /// The "+" button whose headless state machine we drive each frame.
    increment: Button,
    /// The current counter value shown by the label.
    count: i64,
    /// Last known cursor position in physical pixels, tracked for hit-testing
    /// clicks against the stacked bars.
    cursor: (f64, f64),
}

impl WindowedCounter {
    /// Builds the application with a fresh [`UiRuntime`] and the two-button
    /// counter UI, but no window, surface, or GPU device yet (those are created
    /// on `resumed`). The runtime starts at a placeholder viewport matching the
    /// initial window size; [`sync_viewport`](Self::sync_viewport) sets the real
    /// physical extent once the surface exists.
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime: UiRuntime::new(RuntimeId::from_raw(1), Size::new(480.0, 240.0)),
            style_sheet: counter_stylesheet(),
            decrement: Button::new("-", CMD_DECREMENT),
            increment: Button::new("+", CMD_INCREMENT),
            count: 0,
            cursor: (0.0, 0.0),
        }
    }

    /// Maps a click at the tracked cursor position onto a bar and updates the
    /// counter: the lower `.increment` quarter of the window increments, the
    /// `.decrement` quarter above it decrements, and the top `.label` half is
    /// inert. Region boundaries mirror the `counter_stylesheet` percentages.
    fn handle_click(&mut self) {
        let Some(height) = self.surface.as_ref().map(|s| f64::from(s.config().height)) else {
            return;
        };
        let y = self.cursor.1;
        if y >= 0.75 * height {
            self.increment.press();
            if self.increment.release() {
                self.count += 1;
            }
            println!("[+] clicked -> count = {}", self.count);
        } else if y >= 0.5 * height {
            self.decrement.press();
            if self.decrement.release() {
                self.count -= 1;
            }
            println!("[-] clicked -> count = {}", self.count);
        } else {
            return; // Click landed on the label bar: nothing to do.
        }
        // The counter value changed: commit a fresh frame through the runtime so
        // the new label is reconciled, styled, laid out, and painted on the next
        // redraw. (A press that doesn't move focus schedules no frame on its own,
        // so this explicit request is what carries the new value to paint.)
        self.runtime.request_frame(FrameReason::UiCommit);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor.
    ///
    /// [`PointerConverter`] only exposes public builders for `Move`/`Enter`/
    /// `Leave` (a button transition *at a position* is the backend's job), so the
    /// button event is constructed directly — reusing [`PointerConverter::kind`]
    /// to map the winit [`ElementState`] to the domain event kind. The
    /// `CursorMoved` path below uses the converter's `moved` builder end-to-end.
    #[allow(clippy::cast_possible_truncation)]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
    }

    /// Points the runtime's layout viewport at the swapchain's physical extent.
    /// The scene is rasterized 1:1 into the surface, so laying out under a
    /// different viewport would leave the content in a corner. Called after the
    /// surface is created and on every resize. Unlike the old host, the runtime
    /// carries retained state, so this just updates the viewport in place (no
    /// rebuild); the caller requests a frame to actually re-lay-out.
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            let viewport = Size::new(config.width as f32, config.height as f32);
            self.runtime.set_viewport(viewport);
        }
    }

    /// Renders one frame: rebuild the view, run a scheduler-gated frame through
    /// the runtime, read the committed scene, package it, and present it
    /// on-screen via the Vello surface path.
    fn render_frame(&mut self) {
        // --- Stage 1: rebuild the counter view from the current state. --------
        // `run_due_frame` needs a `&View`; the state (`count`, button state)
        // changes on user input, so rebuild it each redraw. Building it first
        // ends the `&self.decrement`/`&self.increment` borrows before the runtime
        // is mutated below.
        let view = build_counter_view(self.count, &self.decrement, &self.increment);
        let sheets = std::slice::from_ref(&self.style_sheet);

        // --- Stage 2: run a frame only if one is due (§3.5). ------------------
        // First paint, input (hover/focus), resize, and counter changes each
        // `request_frame` a reason; `run_due_frame` produces at most one frame
        // per redraw and returns `None` when nothing is pending. We still present
        // the (unchanged) snapshot on a `None` so a bare `RedrawRequested` — e.g.
        // an OS expose — repaints the last committed scene rather than a blank.
        let produced = self.runtime.run_due_frame(&view, sheets);
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
                        let mut renderer =
                            VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
                        // Text is a host resource (like the GPU device): load a
                        // system font and hand its bytes to the renderer so the
                        // counter's labels ("Count: N", "-", "+") rasterize into
                        // real glyphs. With no font found, text is skipped.
                        if let Some(bytes) = load_system_font() {
                            renderer.set_font(bytes);
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
                if let (Some(surface), Some(device)) = (self.surface.as_mut(), self.device.as_ref())
                {
                    surface.resize(device, new_size.width, new_size.height);
                }
                // Keep the layout viewport in step with the new extent (the
                // surface/device borrows above have ended) and schedule a frame
                // so the next redraw re-lays-out under it.
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
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
            WindowEvent::MouseInput { state, button, .. }
                if state == ElementState::Pressed && button == MouseButton::Left =>
            {
                // Dispatch a domain `Down` at the cursor to drive focus through
                // the runtime, then keep the geometric counter update.
                let down = self.pointer_button_event(state);
                self.runtime.handle_pointer(down);
                self.handle_click();
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

/// Loads a system font's raw bytes for the renderer's text path.
///
/// The toolkit never bundles a font (fonts are a host resource, like the GPU
/// device), so the example reads one from a small list of well-known locations
/// across Linux/macOS/Windows and returns the bytes of the first that exists.
/// Returns `None` when none are present, in which case text simply does not
/// render.
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

/// The authored stylesheet: one rule per class, each painting a background +
/// size. Layout is flow (vertical stacking) — `display` is not yet mapped and
/// the property set has no margin/align — so the bars stack top-to-bottom
/// inside the dark `.counter` panel.
///
/// Text color defaults to white in the scene coordinator, and `color` does not
/// reliably inherit to the child text node in the current cascade, so rather
/// than author a per-label text color every bar is given a dark-enough
/// background for white glyphs to read: `.label` is a dark slate, and the red
/// `.decrement` / green `.increment` buttons already carry white text well.
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
    let label = tagged(
        "bar label",
        vec![Text::new(format!("Count: {count}")).into_view_node()],
    );
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

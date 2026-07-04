//! End-to-end **windowed** capstone for the `obscur-kit` facade.
//!
//! Unlike `counter_headless.rs` (no display, no GPU), this example wires the
//! *real* platform backends the `full` feature set unlocks — winit 0.30 for the
//! window and event loop, and Vello/wgpu 29 for the GPU renderer — and drives
//! them through the toolkit's public umbrella crate only (`obscur_kit::*`). It
//! opens a genuine OS window and runs the full GPU render path each frame.
//!
//! Run it on a real machine (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example windowed_counter --features full`
//!
//! # Wiring
//!
//! 1. **Window / event loop** — a winit 0.30 [`ApplicationHandler`]: `resumed`
//!    creates the window and initialises wgpu; `window_event` handles close and
//!    redraw. (The toolkit also ships `obscur_kit::window_winit::backend::
//!    WinitWindowBackend`, the pull-based [`WindowBackend`] port; here we create
//!    the window directly in `resumed`, which is the simplest correct path and
//!    keeps the render-side wiring the focus of the example.)
//! 2. **GPU device** — wgpu `Instance → Adapter → Device → Queue`, handed to
//!    `VelloDevice::new` so the adapter never owns the async `request_adapter`
//!    path (spec §2.7). No surface/swapchain is created — see the limitation
//!    below.
//! 3. **Frame driver** — the deterministic [`HeadlessHost`] runs the real
//!    pipeline (reconcile → style → layout → scene) and exposes the committed
//!    scene as a `SceneSnapshot`. There is no separate "windowed" host; reusing
//!    the headless host as the frame driver is intended.
//! 4. **Submission** — the snapshot is packaged as a `RenderFrame` +
//!    `FrameSubmission` and submitted to the `VelloRenderer` against a
//!    `RenderTarget::Window` sized from the live winit window.
//!
//! # Two honest limitations (why the window shows only a clear color)
//!
//! - **Offscreen render path.** The Vello adapter's `Renderer::render`
//!   rasterizes the scene to an *offscreen* GPU texture (`VelloFrame::
//!   new_offscreen` + `render_to_texture`); it does **not** blit to a winit
//!   swapchain surface — no presentation path is wired into the adapter yet.
//!   So this example runs the full GPU render each frame, but the on-screen
//!   window only ever shows its clear color; the rasterized counter lives in an
//!   offscreen texture that is never presented. Because the render is offscreen,
//!   no wgpu `Surface`/swapchain is needed at all — `Instance → Adapter →
//!   Device → Queue` suffices.
//! - **Empty display list.** Painting needs a CSS selector-matching engine that
//!   is not yet built, so the committed scene's display list is currently empty
//!   and the offscreen texture is a cleared frame.
//!
//! The example's purpose is therefore to prove that the `full` backend graph
//! *links* and that the winit + wgpu + Vello + toolkit wiring compiles and
//! executes end to end — not to display pixels.
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::geometry::Size;
use obscur_kit::headless::HeadlessHost;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::{Renderer, RendererId};
use obscur_kit::render::target::{RenderTarget, WindowTarget};
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_kit::runtime::orchestration::StyleInputs;
use obscur_kit::view::view::{View, ViewNode};
use obscur_kit::widgets::button::Button;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Commands the two buttons fire when activated (interned symbol handles).
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

/// How many frames to render before exiting, so the example terminates on its
/// own rather than spinning a busy redraw loop forever.
const MAX_FRAMES: u32 = 3;

/// The winit application: it owns the window, the GPU-backed renderer, the
/// deterministic frame driver, and the small counter UI state.
struct WindowedCounter {
    /// The live winit window, created lazily in [`ApplicationHandler::resumed`].
    window: Option<Window>,
    /// The Vello/wgpu renderer, attached once the GPU device is created.
    renderer: Option<VelloRenderer>,
    /// The deterministic host that runs the real pipeline each frame.
    host: HeadlessHost,
    /// Styling inputs handed to the host (empty: no external style state here).
    inputs: StyleInputs,
    /// The "-" button (fixed; drives the decrement command symbol).
    decrement: Button,
    /// The "+" button whose headless state machine we drive each frame.
    increment: Button,
    /// The current counter value shown by the label.
    count: i64,
    /// How many frames have been rendered so far (bounds the run).
    frames_rendered: u32,
}

impl WindowedCounter {
    /// Builds the application with a booted [`HeadlessHost`] and the two-button
    /// counter UI, but no window or GPU device yet (those are created on
    /// `resumed`).
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            host: KitBootstrap::new().with_runtime_id(1).headless_host(),
            inputs: StyleInputs::new(),
            decrement: Button::new("-", CMD_DECREMENT),
            increment: Button::new("+", CMD_INCREMENT),
            count: 0,
            frames_rendered: 0,
        }
    }

    /// Renders one frame: advance UI state, build the view, drive the pipeline,
    /// package the scene, and submit it to the Vello renderer (offscreen).
    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        // Nothing to draw until the window and GPU device both exist.
        if self.window.is_none() || self.renderer.is_none() {
            return;
        }

        // --- Stage 1: advance UI state via the button's headless machine. -----
        // A full press + release on an enabled button reports activation.
        self.increment.press();
        if self.increment.release() {
            self.count += 1;
        }

        // --- Stage 2: build the counter view from the components tier. --------
        let view = build_counter_view(self.count, &self.decrement, &self.increment);

        // --- Stage 3: drive the real pipeline and read the committed scene. ---
        let frame = self.host.render(&view, &self.inputs);
        let snapshot = self.host.snapshot();

        // --- Stage 4: derive the viewport + target from the live window. ------
        let window = self.window.as_ref().expect("window created in resumed");
        let physical = window.inner_size();
        // u32 -> f32 for small window extents is intentional and lossless in
        // practice (cast_precision_loss is allowed workspace-wide).
        let physical_size = Size::new(physical.width as f32, physical.height as f32);
        let scale = scale_factor_f32(window.scale_factor());
        let target =
            RenderTarget::Window(WindowTarget::new(physical_size).with_scale_factor(scale));

        // --- Stage 5: package the snapshot into a submission for the port. ----
        let render_frame = RenderFrame::new(snapshot, physical_size);
        let submission = FrameSubmission::new(render_frame);

        // --- Stage 6: submit to the Vello GPU renderer (offscreen render). ----
        let renderer = self.renderer.as_mut().expect("renderer attached in resumed");
        match renderer.render(&submission, &target) {
            Ok(result) => println!(
                "frame {}: pipeline id={} composed={} | vello -> {result:?} (offscreen)",
                self.frames_rendered + 1,
                frame.id().raw(),
                frame.composed(),
            ),
            Err(error) => eprintln!(
                "frame {}: vello render failed: {error:?}",
                self.frames_rendered + 1
            ),
        }

        // --- Stage 7: bound the run so the example exits on its own. ----------
        self.frames_rendered += 1;
        if self.frames_rendered >= MAX_FRAMES {
            println!("rendered {MAX_FRAMES} frame(s); exiting");
            event_loop.exit();
        } else if let Some(window) = self.window.as_ref() {
            // Ask for another frame to keep the loop going up to the bound.
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for WindowedCounter {
    /// Creates the window and the GPU-backed renderer once the platform is ready.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `resumed` can fire more than once (e.g. on Android); build only once.
        if self.window.is_some() {
            return;
        }

        // 1. Create the real winit window.
        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — windowed counter")
            .with_inner_size(LogicalSize::new(480.0, 240.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        // 2. Initialise wgpu (offscreen — no surface/swapchain) and build the
        //    Vello renderer on top of the host-provided device/queue.
        match init_gpu() {
            Ok((device, queue)) => match VelloDevice::new(device, queue) {
                Ok(vello_device) => {
                    let renderer =
                        VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
                    self.renderer = Some(renderer);
                    // Kick off the first frame.
                    window.request_redraw();
                }
                Err(error) => {
                    eprintln!("failed to build the Vello device: {error}");
                    event_loop.exit();
                }
            },
            Err(error) => {
                // Expected in a headless CI environment (no adapter). On a real
                // machine an adapter/device is present.
                eprintln!("no usable GPU adapter/device: {error}");
                event_loop.exit();
            }
        }

        self.window = Some(window);
    }

    /// Routes window events: close requests exit; redraw requests render a frame.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render_frame(event_loop),
            _ => {}
        }
    }
}

/// Initialises a wgpu device/queue for **offscreen** rendering.
///
/// No `Surface`/swapchain is created because the Vello adapter renders to an
/// offscreen texture (see the module-level limitations), so `Instance → Adapter
/// → Device → Queue` is all that is needed. wgpu's async `request_adapter` /
/// `request_device` are driven to completion with `pollster::block_on`.
///
/// # Errors
///
/// Returns an error if no GPU adapter is available (as in a headless
/// environment) or if the device cannot be created.
fn init_gpu() -> Result<(wgpu::Device, wgpu::Queue), Box<dyn std::error::Error>> {
    // wgpu 29: `InstanceDescriptor` has no `Default`, but `Instance` does —
    // `Instance::default()` selects all compiled-in backends.
    let instance = wgpu::Instance::default();
    // wgpu 29: `request_adapter` yields `Result<Adapter, RequestAdapterError>`.
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
    // wgpu 29: `request_device` takes a single `&DeviceDescriptor` and yields
    // `Result<(Device, Queue), RequestDeviceError>`.
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
    Ok((device, queue))
}

/// Narrows a winit `f64` scale factor to the `f32` the render target expects.
#[allow(clippy::cast_possible_truncation)]
fn scale_factor_f32(scale: f64) -> f32 {
    scale as f32
}

/// Builds the counter UI: a [`Container`] grouping a text label and the two
/// buttons' element nodes, returned as a single-root [`View`].
fn build_counter_view(count: i64, decrement: &Button, increment: &Button) -> View {
    let label = Text::new(format!("Count: {count}")).into_view_node();
    let container = Container::new()
        .child(label)
        .with_children(button_nodes(decrement))
        .with_children(button_nodes(increment))
        .into_view_node();
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
    println!("(full backend: winit window + wgpu/Vello GPU renderer, offscreen)\n");

    let event_loop = EventLoop::new()?;
    let mut app = WindowedCounter::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

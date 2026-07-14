//! **Windowed capabilities demo → Vello (GPU, animated).**
//!
//! The live sibling of `capabilities_png.rs` / `capabilities_vello_png.rs`: it
//! renders the *very same* shared [`capabilities`] scene through winit 0.30 +
//! Vello/wgpu 29 into a genuine on-screen window, so the GPU-class effects (the
//! Oklab gradient, the linearRGB blur, the spread shadow) render at **full
//! fidelity** live — and it additionally **animates** the transform-interpolation
//! card (§13).
//!
//! # The animated transform
//!
//! The CSS→motion plumbing is scalar-only today, so the interpolation is driven
//! at the DRIVER level (a clean demo-driver animation, no engine change): each
//! frame the driver blends `rotate(0°) → rotate(90°)` through the motion engine's
//! [`MotionValue::Transform`] decompose→lerp→recompose at an oscillating
//! `t ∈ [0, 1]`, and applies the resulting `[a, b, c, d, e, f]` affine to the
//! `.xform-box` element by layering a per-frame `.xform-box { transform:
//! matrix(...) }` override stylesheet on top of the base sheet (the runtime's
//! real CSS cascade path — a later same-specificity rule wins). The decomposition
//! keeps the square at FULL size through the whole sweep (a naive component-wise
//! matrix lerp would shrink it ~30% at the midpoint); the box rotates 0°→90°→0°.
//!
//! # Device scale & HiDPI
//!
//! The window is **device-scale aware**: the render scale is the display's HiDPI
//! `scale_factor` — **not tied to the window size**. It points the runtime's
//! viewport at `physical_extent / scale` and sets the runtime device scale so the
//! scene is bracketed in a root `scale()` and rasterizes crisply at its authored
//! CSS size. Because the scale is fixed to the display density, **resizing the
//! window reflows the layout** — the UI is *not* zoomed up and down with the
//! window — exactly like a real application.
//!
//! Run it (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example capabilities_window --features full`
//!
//! # Environment note
//!
//! Like the other windowed examples, this needs a display and a usable GPU
//! adapter; in a headless environment it compiles but exits cleanly on `resumed`
//! when no adapter/surface can be acquired (it never panics on that path).
#![forbid(unsafe_code)]

use std::sync::Arc;
use std::time::Instant;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::motion::value::MotionValue;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "capabilities/scene.rs"]
mod capabilities;

/// The animation period in seconds: one full `0° → 90° → 0°` sweep.
const CYCLE_SECS: f32 = 3.2;

/// The initial window size in logical pixels.
const INITIAL_SIZE: Size = Size::new(1180.0, 820.0);

/// The winit application: owns the window, the Vello renderer, the swapchain
/// surface, the runtime, and the animation clock.
struct CapabilitiesWindow {
    window: Option<Arc<Window>>,
    surface: Option<VelloSurface>,
    device: Option<wgpu::Device>,
    renderer: Option<VelloRenderer>,
    runtime: UiRuntime,
    /// The compiled base stylesheet (recomposed each frame with the animated
    /// `.xform-box` override layered on top).
    base_sheet: Stylesheet,
    /// The animation start instant, so `elapsed()` drives the transform sweep.
    start: Instant,
    scale_factor: f64,
}

impl CapabilitiesWindow {
    /// Builds the application with a fresh [`UiRuntime`] wired to the shared scene:
    /// the reactive view builder and the compiled base stylesheet. No window,
    /// surface, or GPU device yet (created on `resumed`).
    fn new() -> Self {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);
        // The static view (no per-frame view rebuild needed — the animation is
        // driven purely through the layered stylesheet, which the runtime restyles).
        runtime.set_root_view(|| capabilities::view(false));
        let base_sheet = compile(capabilities::stylesheet());
        runtime.set_stylesheets(vec![base_sheet.clone()]);

        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            base_sheet,
            start: Instant::now(),
            scale_factor: 1.0,
        }
    }

    /// The base device-pixel scale the UI renders at: the display's HiDPI
    /// [`scale_factor`](Self::scale_factor) — **independent of the window size**.
    ///
    /// The scale is *not* tied to the window width (no fit-to-width zoom): the UI
    /// renders at its natural CSS size for the display density, so widening or
    /// narrowing the window changes the logical viewport and the layout reflows,
    /// exactly like a real application — rather than scaling the whole UI up and
    /// down with the window.
    #[allow(clippy::cast_possible_truncation)]
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    /// Re-syncs the runtime's viewport + device scale to the current swapchain
    /// extent and display density (called after surface creation and on resize).
    ///
    /// The viewport is the swapchain's **physical** extent divided by the device
    /// scale, so layout runs in **logical** CSS pixels that follow the real window
    /// size and reflow on resize.
    #[allow(clippy::cast_possible_truncation)]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            // The kit's standard HiDPI mapping (device scale + logical viewport)
            // in one call — no hand-rolled viewport arithmetic here.
            self.runtime.set_surface(
                Size::new(config.width as f32, config.height as f32),
                self.base_scale(),
            );
        }
    }

    /// The oscillating blend parameter `t ∈ [0, 1]`: a triangle wave over
    /// [`CYCLE_SECS`] so the rotation sweeps `0° → 90° → 0°` and repeats.
    fn blend_t(&self) -> f32 {
        let phase = (self.start.elapsed().as_secs_f32() / CYCLE_SECS).fract();
        if phase < 0.5 { phase * 2.0 } else { 2.0 - phase * 2.0 }
    }

    /// Blends `rotate(0°) → rotate(90°)` through the motion engine at the current
    /// `t` (decompose→lerp→recompose, CSS Transforms §14) and returns the
    /// resulting affine `[a, b, c, d, e, f]` — full-size at every `t`.
    fn animated_matrix(&self) -> [f32; 6] {
        let from = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]); // rotate(0°)
        let to = MotionValue::transform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]); // rotate(90°)
        from.interpolate_checked(&to, self.blend_t())
            .and_then(|v| v.as_transform())
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    /// Compiles the per-frame `.xform-box` override sheet carrying the animated
    /// affine as a `matrix(...)` (transform-origin defaults to the box centre, so
    /// the square rotates in place).
    fn override_sheet(&self) -> Stylesheet {
        let m = self.animated_matrix();
        let css = format!(
            ".xform-box {{ transform: matrix({}, {}, {}, {}, {}, {}); }}",
            m[0], m[1], m[2], m[3], m[4], m[5],
        );
        compile(&css)
    }

    /// Renders one animated frame: push the base + fresh override stylesheets,
    /// run a due reactive frame, and present it on-screen via Vello. Requests the
    /// next redraw so the animation runs continuously.
    #[allow(clippy::cast_possible_truncation)]
    fn render_frame(&mut self) {
        // Layer the animated override on top of the base sheet (later, same
        // specificity → wins), then force a restyle for this frame.
        self.runtime
            .set_stylesheets(vec![self.base_sheet.clone(), self.override_sheet()]);
        self.runtime.request_frame(FrameReason::UiCommit);
        self.runtime.run_due_frame_reactive();
        let snapshot = self.runtime.snapshot();

        let (Some(surface), Some(renderer)) = (self.surface.as_ref(), self.renderer.as_mut())
        else {
            return;
        };
        let physical_size =
            Size::new(surface.config().width as f32, surface.config().height as f32);
        let render_frame = RenderFrame::new(snapshot, physical_size);
        let submission = FrameSubmission::new(render_frame);
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
        // Keep the animation loop alive.
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for CapabilitiesWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — capabilities (Vello, animated)")
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

        match init_gpu(&window) {
            Ok((surface, config, device, queue)) => {
                let vello_surface = VelloSurface::new(surface, config);
                vello_surface.configure(&device);
                match VelloDevice::new(device.clone(), queue) {
                    Ok(vello_device) => {
                        let mut renderer =
                            VelloRenderer::new(RendererId::from_raw(2)).with_device(vello_device);
                        if let Some(bytes) = load_system_font() {
                            wire_text_stack(&mut self.runtime, &mut renderer, bytes);
                        } else {
                            eprintln!("no system font found; text will not render");
                        }
                        self.scale_factor = window.scale_factor();
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        self.sync_viewport();
                        self.start = Instant::now();
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
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.sync_viewport();
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
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
/// Returns an error if the surface cannot be created, no compatible GPU adapter
/// is available, the device cannot be created, or the surface exposes no default
/// configuration.
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

/// Wires the full text stack and registers `bytes` under `sans-serif` on both the
/// resolver and the Vello renderer (identical id↔bytes mapping).
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, bytes: Vec<u8>) {
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

/// Builds the winit event loop and runs the application to completion.
///
/// # Errors
///
/// Propagates any event-loop construction or run error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit capabilities demo → Vello (windowed, animated transform) ==\n");
    let event_loop = EventLoop::new()?;
    let mut app = CapabilitiesWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

#![doc = include_str!("capabilities_window.doc/_module.md")]
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

#[doc = include_str!("capabilities_window.doc/CYCLE_SECS.md")]
const CYCLE_SECS: f32 = 3.2;

#[doc = include_str!("capabilities_window.doc/INITIAL_SIZE.md")]
const INITIAL_SIZE: Size = Size::new(1180.0, 820.0);

#[doc = include_str!("capabilities_window.doc/CapabilitiesWindow.md")]
struct CapabilitiesWindow {
    window: Option<Arc<Window>>,
    surface: Option<VelloSurface>,
    device: Option<wgpu::Device>,
    renderer: Option<VelloRenderer>,
    runtime: UiRuntime,
    #[doc = include_str!("capabilities_window.doc/base_sheet.md")]
    base_sheet: Stylesheet,
    #[doc = include_str!("capabilities_window.doc/start.md")]
    start: Instant,
    scale_factor: f64,
}

impl CapabilitiesWindow {
    #[doc = include_str!("capabilities_window.doc/new.md")]
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

    #[doc = include_str!("capabilities_window.doc/base_scale.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    #[doc = include_str!("capabilities_window.doc/sync_viewport.md")]
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

    #[doc = include_str!("capabilities_window.doc/blend_t.md")]
    fn blend_t(&self) -> f32 {
        let phase = (self.start.elapsed().as_secs_f32() / CYCLE_SECS).fract();
        if phase < 0.5 {
            phase * 2.0
        } else {
            2.0 - phase * 2.0
        }
    }

    #[doc = include_str!("capabilities_window.doc/animated_matrix.md")]
    fn animated_matrix(&self) -> [f32; 6] {
        let from = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]); // rotate(0°)
        let to = MotionValue::transform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]); // rotate(90°)
        from.interpolate_checked(&to, self.blend_t())
            .and_then(|v| v.as_transform())
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    #[doc = include_str!("capabilities_window.doc/override_sheet.md")]
    fn override_sheet(&self) -> Stylesheet {
        let m = self.animated_matrix();
        let css = format!(
            ".xform-box {{ transform: matrix({}, {}, {}, {}, {}, {}); }}",
            m[0], m[1], m[2], m[3], m[4], m[5],
        );
        compile(&css)
    }

    #[doc = include_str!("capabilities_window.doc/render_frame.md")]
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
        let physical_size = Size::new(
            surface.config().width as f32,
            surface.config().height as f32,
        );
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

#[doc = include_str!("capabilities_window.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("capabilities_window.doc/init_gpu.md")]
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

#[doc = include_str!("capabilities_window.doc/wire_text_stack.md")]
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();
    let id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        bytes.clone(),
    );
    renderer.register_font(font_handle(id), bytes);
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

#[doc = include_str!("capabilities_window.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("capabilities_window.doc/load_system_font.md")]
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

#[doc = include_str!("capabilities_window.doc/main.md")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit capabilities demo → Vello (windowed, animated transform) ==\n");
    let event_loop = EventLoop::new()?;
    let mut app = CapabilitiesWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

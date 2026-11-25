#![doc = include_str!("uikit_window.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerButton, PointerButtons, PointerEvent, PointerId};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, FontId, GenericFamily};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::window_winit::conversion::{KeyboardConverter, PointerConverter};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "uikit/scene.rs"]
mod uikit;

use uikit::{KIT_TAB_CMD, MATERIAL_INDEX};

#[doc = include_str!("uikit_window.doc/WHEEL_LINE_STEP.md")]
const WHEEL_LINE_STEP: f32 = 64.0;

#[doc = include_str!("uikit_window.doc/INITIAL_SIZE.md")]
const INITIAL_SIZE: Size = Size::new(1080.0, 900.0);

#[doc = include_str!("uikit_window.doc/UiKitWindow.md")]
struct UiKitWindow {
    #[doc = include_str!("uikit_window.doc/window.md")]
    window: Option<Arc<Window>>,
    #[doc = include_str!("uikit_window.doc/surface.md")]
    surface: Option<VelloSurface>,
    #[doc = include_str!("uikit_window.doc/device.md")]
    device: Option<wgpu::Device>,
    #[doc = include_str!("uikit_window.doc/renderer.md")]
    renderer: Option<VelloRenderer>,
    #[doc = include_str!("uikit_window.doc/runtime.md")]
    runtime: UiRuntime,
    #[doc = include_str!("uikit_window.doc/active_theme.md")]
    active_theme: Rc<RefCell<usize>>,
    #[doc = include_str!("uikit_window.doc/applied_theme.md")]
    applied_theme: usize,
    #[doc = include_str!("uikit_window.doc/cursor.md")]
    cursor: (f64, f64),
    #[doc = include_str!("uikit_window.doc/scale_factor.md")]
    scale_factor: f64,
    #[doc = include_str!("uikit_window.doc/zoom.md")]
    zoom: f32,
    #[doc = include_str!("uikit_window.doc/scroll_y.md")]
    scroll_y: f32,
    #[doc = include_str!("uikit_window.doc/modifiers.md")]
    modifiers: ModifiersState,
}

impl UiKitWindow {
    #[doc = include_str!("uikit_window.doc/new.md")]
    fn new() -> Self {
        let active_theme = Rc::new(RefCell::new(MATERIAL_INDEX));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);

        // One command handler per switcher tab: a click on tab `i` resolves to
        // `KIT_TAB_CMD[i]` and sets the shared index to `i`. The handler only
        // mutates the state bridge (exactly like showcase_window's counter); the
        // window loop reconciles the *stylesheet* swap on the next event, since a
        // handler cannot reach `&mut runtime`.
        for (i, &cmd) in KIT_TAB_CMD.iter().enumerate() {
            let s = active_theme.clone();
            runtime.on_command(cmd, move || {
                *s.borrow_mut() = i;
                println!("[tab] active theme = {i} ({})", uikit::themes()[i].0);
            });
        }
        // The reactive builder rebuilds the identical view for whatever tab is
        // active, with that tab highlighted — the single source of truth.
        {
            let s = active_theme.clone();
            runtime.set_root_view(move || uikit::view(*s.borrow()));
        }
        // Seed the initial (Material) stylesheet; a tab click swaps it live.
        runtime.set_stylesheets(vec![compile(uikit::themes()[MATERIAL_INDEX].1)]);

        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            active_theme,
            applied_theme: MATERIAL_INDEX,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            zoom: 1.0,
            scroll_y: 0.0,
            modifiers: ModifiersState::empty(),
        }
    }

    #[doc = include_str!("uikit_window.doc/base_scale.md")]
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    #[doc = include_str!("uikit_window.doc/effective_scale.md")]
    fn effective_scale(&self) -> f32 {
        self.base_scale() * self.zoom
    }

    #[doc = include_str!("uikit_window.doc/content_height_logical.md")]
    fn content_height_logical(&self) -> f32 {
        self.runtime
            .inspect_snapshot()
            .nodes
            .iter()
            .filter_map(|node| node.bounds)
            .map(|rect| rect.origin.y + rect.size.height)
            .fold(0.0_f32, f32::max)
    }

    #[doc = include_str!("uikit_window.doc/max_scroll_y.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn max_scroll_y(&self) -> f32 {
        let Some(surface) = self.surface.as_ref() else {
            return 0.0;
        };
        let physical_height = surface.config().height as f32;
        let content_height = self.content_height_logical() * self.effective_scale();
        (content_height - physical_height).max(0.0)
    }

    #[doc = include_str!("uikit_window.doc/apply_scroll.md")]
    fn apply_scroll(&mut self) {
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
        let offset = self.scroll_y / self.effective_scale();
        self.runtime.set_content_offset(0.0, offset);
    }

    #[doc = include_str!("uikit_window.doc/pointer_button_event.md")]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        let converter = PointerConverter::new();
        let position = winit::dpi::PhysicalPosition::new(self.cursor.0, self.cursor.1);
        let button = PointerButton::Primary;
        let mut held = PointerButtons::new();
        match state {
            ElementState::Pressed => {
                held.press(button);
                converter.pressed(PointerId::MOUSE, position, button, held)
            }
            ElementState::Released => converter.released(PointerId::MOUSE, position, button, held),
        }
    }

    #[doc = include_str!("uikit_window.doc/sync_viewport.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            // The kit's standard HiDPI mapping (device scale + logical viewport)
            // in one call — no hand-rolled viewport arithmetic here.
            self.runtime.set_surface(
                Size::new(config.width as f32, config.height as f32),
                self.effective_scale(),
            );
        }
        // The scale/zoom just changed, so the scrollable range may have too —
        // re-clamp and re-push the content offset.
        self.apply_scroll();
    }

    #[doc = include_str!("uikit_window.doc/log_scale_sanity.md")]
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

    #[doc = include_str!("uikit_window.doc/sync_theme_stylesheet.md")]
    fn sync_theme_stylesheet(&mut self) -> bool {
        let active = *self.active_theme.borrow();
        if active == self.applied_theme {
            return false;
        }
        let (name, css) = uikit::themes()[active];
        self.runtime.set_stylesheets(vec![compile(css)]);
        self.applied_theme = active;
        self.runtime.request_frame(FrameReason::UiCommit);
        println!("[theme] swapped stylesheet → {name} (index {active})");
        true
    }

    #[doc = include_str!("uikit_window.doc/handle_zoom_shortcut.md")]
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

    #[doc = include_str!("uikit_window.doc/render_frame.md")]
    fn render_frame(&mut self) {
        // Run the due frame for its side effects; the per-present firehose is
        // intentionally silent — meaningful activity is logged as EVENTS
        // (`[tab]`, `[theme]`, `[zoom]`, `[resize]`), not once per presented frame.
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
    }
}

impl ApplicationHandler for UiKitWindow {
    #[doc = include_str!("uikit_window.doc/resumed.md")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — UI Kit theme switcher (Vello, interactive)")
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
                        self.log_scale_sanity();
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
                println!("[resize] {}×{} px", new_size.width, new_size.height);
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(
                    PointerId::MOUSE,
                    position,
                    obscur_kit::interaction::pointer::PointerButtons::new(),
                );
                self.runtime.handle_pointer(moved);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                // Real domain pointer path: `handle_pointer` hit-tests, and on a
                // release completing a click it resolves the tab's `on-click` to
                // `KIT_TAB_CMD[i]` and runs its handler (which sets the shared
                // `active_theme`). We then reconcile the stylesheet swap here.
                let event = self.pointer_button_event(state);
                let dispatch = self.runtime.handle_pointer(event);
                if let Some(command) = dispatch.command() {
                    println!("[click] resolved command {command:?}");
                }
                self.sync_theme_stylesheet();
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
                // Ctrl+zoom shortcuts, handled on key-down; there is no editable
                // widget in this scene, so no further key forwarding is needed.
                if event.state == ElementState::Pressed
                    && self.modifiers.control_key()
                    && self.handle_zoom_shortcut(&event.logical_key)
                {
                    return;
                }
                // Forward other keys through the real domain path (harmless no-op
                // for this scene, but keeps the wiring honest).
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        self.runtime
                            .handle_key(KeyEvent::new(physical, logical, pressed));
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

#[doc = include_str!("uikit_window.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("uikit_window.doc/init_gpu.md")]
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

#[doc = include_str!("uikit_window.doc/wire_text_stack.md")]
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

#[doc = include_str!("uikit_window.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("uikit_window.doc/load_system_font.md")]
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

#[doc = include_str!("uikit_window.doc/load_fallback_font.md")]
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

#[doc = include_str!("uikit_window.doc/main.md")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit imported UI Kit → Vello (windowed, interactive theme switcher) ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)");
    println!("click a theme tab to swap the stylesheet live · Ctrl +/-/0 to zoom\n");

    let event_loop = EventLoop::new()?;
    let mut app = UiKitWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

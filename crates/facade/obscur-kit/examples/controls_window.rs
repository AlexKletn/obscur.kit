#![doc = include_str!("controls_window.doc/_module.md")]
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
use obscur_kit::text::ime::{CompositionEvent, Preedit};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::window_winit::conversion::{KeyboardConverter, PointerConverter};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "controls/scene.rs"]
mod controls;

use controls::{
    CMD_BTN, CMD_CHK, CMD_FIELD, CMD_SEL_TOGGLE, CMD_TRI, GalleryState, RADIO_COUNT, RADIO_LABELS,
    SELECT_COUNT, SELECT_LABELS, radio_command, select_option_command,
};

#[doc = include_str!("controls_window.doc/WHEEL_LINE_STEP.md")]
const WHEEL_LINE_STEP: f32 = 64.0;
#[doc = include_str!("controls_window.doc/INITIAL_SIZE.md")]
const INITIAL_SIZE: Size = Size::new(760.0, 720.0);

#[doc = include_str!("controls_window.doc/ControlsWindow.md")]
struct ControlsWindow {
    window: Option<Arc<Window>>,
    surface: Option<VelloSurface>,
    device: Option<wgpu::Device>,
    renderer: Option<VelloRenderer>,
    runtime: UiRuntime,
    cursor: (f64, f64),
    scale_factor: f64,
    zoom: f32,
    scroll_y: f32,
    modifiers: ModifiersState,
}

impl ControlsWindow {
    #[doc = include_str!("controls_window.doc/new.md")]
    fn new() -> Self {
        let state = Rc::new(RefCell::new(GalleryState::default()));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);

        // Button → bump the click counter.
        {
            let s = state.clone();
            runtime.on_command(CMD_BTN, move || {
                let mut st = s.borrow_mut();
                st.click_count += 1;
                st.last_event = format!("Button clicked (#{})", st.click_count);
            });
        }
        // Checkbox → real tri-state/toggle state machine.
        {
            let s = state.clone();
            runtime.on_command(CMD_CHK, move || {
                let mut st = s.borrow_mut();
                st.check.toggle();
                let on = st.check.is_checked();
                st.last_event = format!("Checkbox → {}", if on { "checked" } else { "unchecked" });
            });
        }
        {
            let s = state.clone();
            runtime.on_command(CMD_TRI, move || {
                let mut st = s.borrow_mut();
                st.tri.toggle();
                st.last_event = format!("Tri-state → {}", tri_word(&st.tri));
            });
        }
        // Radio group — one handler per option, sharing the selected index.
        for i in 0..RADIO_COUNT {
            let s = state.clone();
            runtime.on_command(radio_command(i), move || {
                let mut st = s.borrow_mut();
                st.radio = i;
                st.last_event = format!("Radio → {}", RADIO_LABELS[i]);
            });
        }
        // Select header — toggle the real popup open/closed.
        {
            let s = state.clone();
            runtime.on_command(CMD_SEL_TOGGLE, move || {
                let mut st = s.borrow_mut();
                st.select.toggle_open();
                st.last_event = format!(
                    "Select {}",
                    if st.select.is_open() {
                        "opened"
                    } else {
                        "closed"
                    }
                );
            });
        }
        // Select options — one handler per option; select() closes the popup.
        for i in 0..SELECT_COUNT {
            let s = state.clone();
            runtime.on_command(select_option_command(i), move || {
                let mut st = s.borrow_mut();
                st.select.select(i);
                st.last_event = format!("Selected {}", SELECT_LABELS[i]);
            });
        }
        // Input — per-keystroke text mirrored into state from the editing session.
        {
            let s = state.clone();
            runtime.on_text_input(CMD_FIELD, move |text| {
                let mut st = s.borrow_mut();
                text.clone_into(&mut st.input);
                st.last_event = format!("Typed: {text:?}");
            });
        }
        // The reactive view builder + the compiled stylesheet.
        {
            let s = state.clone();
            runtime.set_root_view(move || controls::view(&s.borrow()));
        }
        runtime.set_stylesheets(vec![compile(controls::stylesheet())]);

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
        }
    }

    #[doc = include_str!("controls_window.doc/base_scale.md")]
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    fn effective_scale(&self) -> f32 {
        self.base_scale() * self.zoom
    }

    #[doc = include_str!("controls_window.doc/content_height_logical.md")]
    fn content_height_logical(&self) -> f32 {
        self.runtime
            .inspect_snapshot()
            .nodes
            .iter()
            .filter_map(|node| node.bounds)
            .map(|rect| rect.origin.y + rect.size.height)
            .fold(0.0_f32, f32::max)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn max_scroll_y(&self) -> f32 {
        let Some(surface) = self.surface.as_ref() else {
            return 0.0;
        };
        let physical_height = surface.config().height as f32;
        let content_height = self.content_height_logical() * self.effective_scale();
        (content_height - physical_height).max(0.0)
    }

    fn apply_scroll(&mut self) {
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
        let offset = self.scroll_y / self.effective_scale();
        self.runtime.set_content_offset(0.0, offset);
    }

    #[doc = include_str!("controls_window.doc/pointer_button_event.md")]
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

    #[doc = include_str!("controls_window.doc/sync_viewport.md")]
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
        self.apply_scroll();
    }

    #[doc = include_str!("controls_window.doc/handle_zoom_shortcut.md")]
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
        if text.as_str() == "0" {
            self.scroll_y = 0.0;
        }
        self.zoom = new_zoom;
        self.sync_viewport();
        self.runtime.request_frame(FrameReason::WindowChange);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        true
    }

    #[doc = include_str!("controls_window.doc/render_frame.md")]
    fn render_frame(&mut self) {
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

impl ApplicationHandler for ControlsWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — interactive control gallery (Vello)")
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
                        self.scale_factor = window.scale_factor();
                        self.surface = Some(vello_surface);
                        self.device = Some(device);
                        self.renderer = Some(renderer);
                        self.window = Some(window);
                        self.sync_viewport();
                        self.runtime.request_frame(FrameReason::UiCommit);
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                        println!(
                            "controls window ready — hover, click, type, toggle, open the select."
                        );
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
                if event.state == ElementState::Pressed
                    && self.modifiers.control_key()
                    && self.handle_zoom_shortcut(&event.logical_key)
                {
                    return;
                }
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

#[doc = include_str!("controls_window.doc/tri_word.md")]
fn tri_word(tri: &obscur_kit::widgets::checkbox::Checkbox) -> &'static str {
    if tri.is_indeterminate() {
        "mixed"
    } else if tri.is_checked() {
        "checked"
    } else {
        "unchecked"
    }
}

#[doc = include_str!("controls_window.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("controls_window.doc/init_gpu.md")]
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

#[doc = include_str!("controls_window.doc/wire_text_stack.md")]
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

#[doc = include_str!("controls_window.doc/font_handle.md")]
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

#[doc = include_str!("controls_window.doc/load_system_font.md")]
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

#[doc = include_str!("controls_window.doc/load_fallback_font.md")]
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

#[doc = include_str!("controls_window.doc/main.md")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit interactive control gallery → Vello (windowed) ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, live pseudo-classes)\n");

    let event_loop = EventLoop::new()?;
    let mut app = ControlsWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

//! **Interactive windowed imported-UI-Kit theme switcher → Vello (GPU).**
//!
//! The live sibling of `uikit_png.rs` / `uikit_vello_png.rs`: it renders the *very
//! same* shared [`uikit`] view (the top theme-switcher bar over the component
//! gallery) through winit 0.30 + Vello/wgpu into a genuine on-screen window — and
//! makes the switcher **clickable**. Clicking a theme tab swaps that theme's
//! stylesheet and rebuilds the identical view *live*, so the whole gallery re-skins
//! in place at full GPU fidelity.
//!
//! Because this is the GPU backend, the gradient/shadow-heavy themes render for
//! real, not the CPU approximation:
//!
//! - the **Skeuomorphic** tab (index 4) paints its `linear-gradient(#light,#dark)`
//!   faces as real smooth ramps and its `inset`/outset `box-shadow` gloss + cast
//!   shadows as real Gaussian blurs (the CPU PNG shows a flat first-stop fill and a
//!   flat offset rect);
//! - the **Material** tab (index 1, the default) paints its tonal fills, elevation
//!   shadows, and the `mini-cover` gradient at full fidelity;
//! - the remaining six tabs are the neutral placeholder stylesheet for now — a
//!   click still swaps to them live (they just look plain, by design).
//!
//! The view structure is *identical* across every tab (that is the whole point of
//! the one-view/many-stylesheets architecture); a tab click is therefore a **pure**
//! stylesheet swap plus a rebuild of the same `uikit::view(i)` with tab `i`
//! highlighted.
//!
//! # Fit-to-height & zoom
//!
//! The gallery is a tall fixed design (~[`GALLERY_HEIGHT`] logical px). The window
//! is **device-scale aware**: it picks a base *fit* scale so the whole gallery
//! height is visible (`physical_height / GALLERY_HEIGHT`, never upscaled past the
//! display's HiDPI `scale_factor`), sets that as [`UiRuntime::set_device_scale`],
//! and points the viewport at `physical_extent / effective_scale` — so layout runs
//! in logical CSS pixels and the whole scene is bracketed by a root `scale(s, s)`
//! and rasterizes crisply. A user **zoom** multiplies on top: `Ctrl+=` / `Ctrl++`
//! zoom in (cap 500%), `Ctrl+-` zooms out (min 30%), `Ctrl+0` resets. The fit is
//! recomputed on resize / scale-factor change / zoom. Pointer coordinates map
//! through the device scale automatically (the runtime divides by it), so tab
//! hit-testing is correct at any fit or zoom.
//!
//! Run it (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example uikit_window --features full`
//!
//! Click the **Material** vs **Skeuomorphic** tabs to swap the look live;
//! `Ctrl` +/-/0 to zoom.
//!
//! # Environment note
//!
//! Like `showcase_window.rs`, this needs a display and a usable GPU adapter; in a
//! headless environment it compiles but exits cleanly on `resumed` when no
//! adapter/surface can be acquired (it never panics on that path).
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::RuntimeId;
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
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
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "uikit/scene.rs"]
mod uikit;

use uikit::{KIT_TAB_CMD, MATERIAL_INDEX};

/// The gallery's fixed logical design height (matches the `uikit_*_png` viewport,
/// 1280×2080). The window fits this whole height into view by choosing a base
/// device scale of `physical_height / GALLERY_HEIGHT` (with a little margin).
const GALLERY_HEIGHT: f32 = 2080.0;

/// A small margin factor so the fitted gallery does not sit edge-to-edge: the fit
/// targets `GALLERY_HEIGHT * FIT_MARGIN` logical px of viewport height.
const FIT_MARGIN: f32 = 1.04;

/// The initial window size in *logical* pixels. Chosen with roughly the gallery's
/// tall aspect so the fit-to-height render fills the window with the full gallery
/// visible from the first frame; the window is freely resizable.
const INITIAL_SIZE: Size = Size::new(940.0, 1000.0);

/// The winit application: owns the window, the GPU-backed Vello renderer, the
/// swapchain surface, the embeddable [`UiRuntime`], and the shared `active_theme`
/// index the clickable switcher drives.
struct UiKitWindow {
    /// The live winit window (an [`Arc`] so wgpu can build a `Surface<'static>`),
    /// created lazily in [`ApplicationHandler::resumed`].
    window: Option<Arc<Window>>,
    /// The on-screen swapchain surface, created alongside the window.
    surface: Option<VelloSurface>,
    /// The wgpu device, cached so resizes can reconfigure the surface.
    device: Option<wgpu::Device>,
    /// The Vello/wgpu renderer, attached once the GPU device is created.
    renderer: Option<VelloRenderer>,
    /// The embeddable runtime that owns the retained tree, scene, scheduler, and
    /// live interaction state. Its reactive view builder reads the shared
    /// `active_theme` to rebuild `uikit::view(i)`; its per-tab command handlers
    /// mutate that same index.
    runtime: UiRuntime,
    /// The active theme index, shared behind an `Rc<RefCell<_>>` by the per-tab
    /// command handlers (which set it on a click) and the reactive view builder
    /// (which reads it back to rebuild the highlighted view) — the state bridge.
    active_theme: Rc<RefCell<usize>>,
    /// The theme index whose stylesheet is currently installed on the runtime, so
    /// the window can detect a tab-driven change and swap the stylesheet (which a
    /// command handler cannot reach). Starts equal to `active_theme`.
    applied_theme: usize,
    /// Last known cursor position in physical pixels, so a `MouseInput` (which
    /// carries no position) can be dispatched as a pointer event at the cursor.
    cursor: (f64, f64),
    /// The window's current HiDPI device-pixel ratio (winit `scale_factor`).
    scale_factor: f64,
    /// The user zoom factor (Ctrl+= / Ctrl+- / Ctrl+0), multiplied with the fit
    /// scale to form the effective device scale. `1.0` is 100%.
    zoom: f32,
    /// The modifier keys currently held, so the keyboard handler can recognise the
    /// Ctrl+zoom chords.
    modifiers: ModifiersState,
}

impl UiKitWindow {
    /// Builds the application with a fresh [`UiRuntime`] wired to the shared UI-Kit
    /// scene: the reactive view builder (`uikit::view(active_theme)`), a command
    /// handler per switcher tab, and the initial (Material) stylesheet. No window,
    /// surface, or GPU device yet (created on `resumed`).
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
            modifiers: ModifiersState::empty(),
        }
    }

    /// The base **fit** scale that makes the whole gallery height visible: the
    /// swapchain's physical height divided by the (slightly margined) gallery
    /// height, never upscaled past the display's HiDPI `scale_factor` (so a large
    /// window renders at native crispness rather than blowing the content up).
    #[allow(clippy::cast_possible_truncation)]
    fn fit_scale(&self) -> f32 {
        let scale_factor = self.scale_factor as f32;
        let Some(surface) = self.surface.as_ref() else {
            return scale_factor;
        };
        let physical_height = surface.config().height as f32;
        let base = physical_height / (GALLERY_HEIGHT * FIT_MARGIN);
        base.min(scale_factor).max(0.05)
    }

    /// The effective device-pixel scale the runtime renders at: the fit-to-height
    /// scale times the user `zoom`. The viewport is the physical extent divided by
    /// this, so at `zoom == 1.0` the whole gallery height fits the window.
    fn effective_scale(&self) -> f32 {
        self.fit_scale() * self.zoom
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor (physical
    /// px; the runtime divides by the device scale to logical space).
    #[allow(clippy::cast_possible_truncation)]
    fn pointer_button_event(&self, state: ElementState) -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerConverter::kind(state),
            Point::new(self.cursor.0 as f32, self.cursor.1 as f32),
        )
    }

    /// Re-syncs the runtime's layout viewport and device scale to the current
    /// swapchain extent, fit-to-height scale, and user zoom. Called after surface
    /// creation, on resize, on a scale-factor change, and after a zoom change.
    #[allow(clippy::cast_possible_truncation)]
    fn sync_viewport(&mut self) {
        if let Some(surface) = self.surface.as_ref() {
            let config = surface.config();
            let scale = self.effective_scale();
            let viewport = Size::new(config.width as f32 / scale, config.height as f32 / scale);
            self.runtime.set_viewport(viewport);
            self.runtime.set_device_scale(scale);
        }
    }

    /// Reconciles the installed stylesheet with the shared `active_theme` a tab
    /// handler may have just changed: if it differs from [`applied_theme`], swap in
    /// `themes()[i]` (a command handler cannot reach `&mut runtime`, so this is the
    /// window's job) and request a structural frame so the reactive builder rebuilds
    /// the re-skinned view. Returns `true` when a swap happened.
    ///
    /// [`applied_theme`]: Self::applied_theme
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

    /// Handles a Ctrl+zoom shortcut, returning `true` when `key` was a zoom chord
    /// (so the caller swallows it): `Ctrl+=`/`Ctrl++` zoom in (×1.1, cap 5.0),
    /// `Ctrl+-` zoom out (÷1.1, floor 0.3), `Ctrl+0` reset. A change re-syncs the
    /// viewport + device scale and requests a redraw.
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
        self.zoom = new_zoom;
        self.sync_viewport();
        self.runtime.request_frame(FrameReason::WindowChange);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        println!("[zoom] {:.0}% (device scale {:.2})", self.zoom * 100.0, self.effective_scale());
        true
    }

    /// Renders one frame: run a scheduler-gated reactive frame (rebuilding the view
    /// from the active theme when due), read the committed scene, and present it via
    /// Vello.
    fn render_frame(&mut self) {
        let produced = self.runtime.run_due_frame_reactive();
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
            Ok(result) => {
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
                if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }
    }
}

impl ApplicationHandler for UiKitWindow {
    /// Creates the window, swapchain surface, and GPU-backed Vello renderer once
    /// the platform is ready, and wires the host text stack.
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
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
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
                        self.runtime.handle_key(KeyEvent::new(physical, logical, pressed));
                    }
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through the
/// real `obscur-css` frontend (identical to `uikit_vello_png.rs`).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Initialises a wgpu swapchain surface + device/queue for on-screen rendering
/// from an `Arc<Window>` (mirrors `showcase_window.rs`).
///
/// # Errors
///
/// Returns an error if the surface cannot be created, no compatible GPU adapter is
/// available (as in a headless environment), the device cannot be created, or the
/// surface exposes no supported default configuration.
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

/// Wires the full multi-font fallback stack onto the runtime and the Vello renderer
/// (identical to `showcase_window.rs`): the primary under `sans-serif` (id 0) plus
/// a best-effort CJK fallback under the same family (id 1), so resolution is by
/// `cmap` COVERAGE. The renderer receives the same id↔bytes assignment.
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

    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(HarfrustShaper::without_font()));
}

/// The scene font handle (a `u32`) for a resolved [`FontId`].
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes, scanning well-known Linux/macOS/Windows
/// locations and returning the first that exists, or `None`.
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

/// Loads a second, CJK-covering fallback font's bytes (best effort).
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

/// Builds the winit event loop and runs the application to completion.
///
/// # Errors
///
/// Propagates any event-loop construction or run error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit imported UI Kit → Vello (windowed, interactive theme switcher) ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)");
    println!("click a theme tab to swap the stylesheet live · Ctrl +/-/0 to zoom\n");

    let event_loop = EventLoop::new()?;
    let mut app = UiKitWindow::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}

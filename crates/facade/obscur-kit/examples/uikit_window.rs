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
//! # Device scale, wheel scroll & zoom
//!
//! The window is **device-scale aware**: the render scale is the display's HiDPI
//! `scale_factor` — **not tied to the window size**. It hands the runtime its
//! swapchain via [`UiRuntime::set_surface`], which points the viewport at
//! `physical_extent / effective_scale` and sets the device scale to that same
//! factor — the runtime then brackets the whole scene in a root `scale(s, s)`
//! transform, so vectors, text, and gradients rasterize crisply at their authored
//! CSS size. Because the scale is fixed to the display density, **resizing the
//! window reflows the layout** (the gallery divides the real width; the UI is *not*
//! zoomed up and down with the window) — exactly like a real application. Any
//! content taller than the window is reached by **scrolling**: the mouse wheel
//! drives a vertical `scroll_y`, pushed to the runtime as
//! [`UiRuntime::set_content_offset`] (a root `translate` composed with the device
//! scale). A user **zoom** multiplies on top: `Ctrl+=` / `Ctrl++` zoom in (cap
//! 500%), `Ctrl+-` zooms out (min 30%), `Ctrl+0` resets zoom **and** scroll. The
//! device scale + scroll clamp are recomputed on resize / scale-factor change /
//! zoom. Pointer coordinates map through the device scale + scroll offset
//! automatically (the runtime divides by the scale and adds the offset back), so
//! tab hit-testing is correct at any scale, zoom, or scroll position.
//!
//! Run it (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example uikit_window --features full`
//!
//! Click the **Material** vs **Skeuomorphic** tabs to swap the look live;
//! **mouse wheel** scrolls vertically; `Ctrl` +/-/0 to zoom / reset.
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
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "uikit/scene.rs"]
mod uikit;

use uikit::{KIT_TAB_CMD, MATERIAL_INDEX};

/// One wheel *line* of scroll, in logical content px, mapped to a comfortable step
/// (a `LineDelta` of 1.0 scrolls this much content). `PixelDelta` deltas are used
/// as-is (already in physical px).
const WHEEL_LINE_STEP: f32 = 64.0;

/// The initial window size in *logical* pixels. A wide, moderately tall window so
/// the gallery renders at readable size from the first frame and there is room to
/// wheel-scroll the rest; freely resizable (resize reflows the layout).
const INITIAL_SIZE: Size = Size::new(1080.0, 900.0);

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
    /// The user zoom factor (Ctrl+= / Ctrl+- / Ctrl+0), multiplied with the
    /// display density to form the effective device scale. `1.0` is 100%.
    zoom: f32,
    /// The vertical scroll offset in **physical** px (mouse-wheel driven), clamped
    /// to `0 ..= max(0, content_height*effective_scale - physical_height)`. Pushed
    /// to the runtime as a logical content offset (`scroll_y / effective_scale`).
    scroll_y: f32,
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
            scroll_y: 0.0,
            modifiers: ModifiersState::empty(),
        }
    }

    /// The base device-pixel scale the UI renders at: the display's HiDPI
    /// [`scale_factor`](Self::scale_factor) — **independent of the window size**.
    ///
    /// The scale is *not* tied to the window width (no fit-to-width zoom): the
    /// gallery renders at its natural CSS size for the display density, so widening
    /// or narrowing the window changes the logical viewport and the layout reflows
    /// (the gallery divides the available width), exactly like a real application —
    /// rather than scaling the whole UI up and down. The user `zoom` composes on
    /// top for an explicit magnify.
    fn base_scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    /// The effective device-pixel scale the runtime renders at: the display
    /// density times the user `zoom`. The viewport is the physical extent divided
    /// by this, so layout runs in logical CSS pixels that follow the real window
    /// size and reflow on resize.
    fn effective_scale(&self) -> f32 {
        self.base_scale() * self.zoom
    }

    /// The tallest content extent actually laid out this frame, in **logical**
    /// px — the largest bottom edge across the tree, read from the runtime's live
    /// [`inspect_snapshot`](UiRuntime::inspect_snapshot). Used for the scroll
    /// range, so scrolling reflects the real content rather than a fixed design
    /// height.
    fn content_height_logical(&self) -> f32 {
        self.runtime
            .inspect_snapshot()
            .nodes
            .iter()
            .filter_map(|node| node.bounds)
            .map(|rect| rect.origin.y + rect.size.height)
            .fold(0.0_f32, f32::max)
    }

    /// The maximum vertical scroll in **physical** px: how far past the bottom of
    /// the window the real content extends. `0` when the whole content already fits
    /// (so `scroll_y` simply clamps to `0`, harmless).
    #[allow(clippy::cast_possible_truncation)]
    fn max_scroll_y(&self) -> f32 {
        let Some(surface) = self.surface.as_ref() else {
            return 0.0;
        };
        let physical_height = surface.config().height as f32;
        let content_height = self.content_height_logical() * self.effective_scale();
        (content_height - physical_height).max(0.0)
    }

    /// Re-clamps `scroll_y` into `0 ..= max_scroll_y()` and pushes the resulting
    /// **logical** content offset (`scroll_y / effective_scale`) to the runtime, so
    /// a physical wheel step scrolls a consistent on-screen amount at any scale.
    fn apply_scroll(&mut self) {
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll_y());
        let offset = self.scroll_y / self.effective_scale();
        self.runtime.set_content_offset(0.0, offset);
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
    /// swapchain extent, display density, and user zoom. Called after surface
    /// creation, on resize, on a scale-factor change, and after a zoom change.
    ///
    /// The viewport is the swapchain's **physical** extent divided by the effective
    /// scale, so layout runs in **logical** CSS pixels; the runtime's device scale
    /// is set to that same factor, so the composed scene is bracketed by a root
    /// `scale(s, s)` and rasterizes crisply at physical resolution.
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

    /// Prints the computed fit/effective scale and content bounds once so the scale
    /// math is inspectable from the logs even though the window itself is not
    /// visible in a smoke run.
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
    /// `Ctrl+-` zoom out (÷1.1, floor 0.3), `Ctrl+0` reset zoom **and** scroll. A
    /// change re-syncs the viewport + device scale (+ scroll clamp) and requests a
    /// redraw.
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
        println!("[zoom] {:.0}% (device scale {:.2})", self.zoom * 100.0, self.effective_scale());
        true
    }

    /// Renders one frame: run a scheduler-gated reactive frame (rebuilding the view
    /// from the active theme when due), read the committed scene, and present it via
    /// Vello.
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
                println!("[scroll] y = {:.0} px (max {:.0})", self.scroll_y, self.max_scroll_y());
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

    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
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

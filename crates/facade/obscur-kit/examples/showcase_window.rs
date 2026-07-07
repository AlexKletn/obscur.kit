//! **Windowed capability showcase → Vello (GPU, interactive).**
//!
//! The GPU sibling of `showcase_png.rs`: it renders the *very same* shared
//! [`showcase`] scene (grid + flex layout, gradients, borders, radius, shadows,
//! blur, group opacity, transforms, a `background-image` + an `<img>` primitive,
//! and real shaped text driving the [`Button`]/[`Checkbox`]/[`Input`] widgets) —
//! but through winit 0.30 + Vello/wgpu 29 into a genuine on-screen window, so the
//! GPU-class effects render at **full fidelity** instead of the CPU
//! approximation. Concretely, on this backend:
//!
//! - `linear-gradient` / `radial-gradient` / `conic-gradient` paint real smooth
//!   ramps (the CPU PNG shows only the first stop);
//! - `box-shadow` and `filter: drop-shadow(...)` paint real Gaussian-blurred
//!   shadows (the CPU PNG paints a flat offset rect);
//! - `opacity: 0.5` composites a real translucent group layer;
//! - `transform: rotate()/scale()`, borders, radius, images, and shaped text all
//!   render as on the CPU, but presented live.
//!
//! It is also **interactive**, driving the shared [`ShowcaseState`] through the
//! runtime's real domain input + command path and rebuilding the view reactively:
//!
//! - clicking the **Button** ("Click me") fires its `EVENT_CLICK` command, whose
//!   handler bumps `state.counter` — the toolbar's `counter = N` text updates;
//! - clicking the **Checkbox** fires its change command, toggling `state.checked`;
//! - the **Input** is editable: clicking it seats the caret, and typing / IME
//!   flows winit → converters → `UiRuntime` → the runtime-owned editing session,
//!   whose per-keystroke `on_text_input` handler mirrors the resulting text into
//!   `state.input_text` so the reactive builder repaints the field.
//!
//! # HiDPI & zoom
//!
//! The window is **device-scale aware**: it lays out in *logical* CSS pixels and
//! renders at the display's *physical* resolution, so the UI is never tiny on a
//! HiDPI screen. It reads winit's `scale_factor` (and tracks `ScaleFactorChanged`)
//! and points the runtime's viewport at `physical_extent / effective_scale` while
//! setting `UiRuntime::set_device_scale(effective_scale)` — the runtime then
//! brackets the whole scene in a root `scale(s, s)` transform, so vectors, text,
//! and gradients scale crisply. A user **zoom** multiplies on top: `Ctrl+=` /
//! `Ctrl++` zoom in (cap 500%), `Ctrl+-` zooms out (min 30%), and `Ctrl+0` resets
//! to 100%. The initial window is 1000×700 *logical* pixels.
//!
//! Run it (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example showcase_window --features full`
//!
//! # Wiring
//!
//! This is a thin driver over the shared scene and the existing
//! runtime/converter/Vello APIs — it adds no rendering logic of its own. The
//! window/device/surface/present path mirrors `windowed_counter.rs`; the scene,
//! stylesheet, and image-asset setup mirror `showcase_png.rs`. The image asset is
//! staged into a sandbox temp dir the runtime's filesystem loader is rooted at
//! (via [`KitBootstrap::install_image_resources`]), so both the
//! `background-image: url("gallery.png")` card and the `<img>` primitive resolve
//! to real decoded pixels. Text is a host resource: a system font (plus a
//! best-effort CJK fallback) is loaded off disk and wired as the full itemize →
//! resolve-by-coverage → shape stack, with the same id↔bytes assignment handed to
//! the Vello renderer.
//!
//! # Environment note
//!
//! Like `windowed_counter.rs`, this example needs a display and a usable GPU
//! adapter; in a headless environment it compiles but exits cleanly on
//! `resumed` when no adapter/surface can be acquired (it never panics on that
//! path). On a real machine it opens a window and presents the showcase live.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
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
use winit::event::{ElementState, Ime, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, ModifiersState, PhysicalKey as WinitPhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[path = "showcase/scene.rs"]
mod showcase;

use showcase::ShowcaseState;

/// The command symbols the shared scene binds to its interactive widgets. The
/// scene keeps these private (both examples subset it), so the windowed driver
/// mirrors the exact same raw values here to register the matching handlers — the
/// symbols the runtime resolves a widget activation to.
const CMD_BUTTON: InternedSymbol = InternedSymbol::from_raw(0x5001);
const CMD_CHECKBOX: InternedSymbol = InternedSymbol::from_raw(0x5002);
const CMD_INPUT: InternedSymbol = InternedSymbol::from_raw(0x5003);

/// The initial (and layout) window size in logical pixels; matches the CPU PNG
/// viewport so the two examples frame the identical scene. The window is
/// resizable — resize reconfigures the swapchain and re-lays-out under the new
/// physical extent.
const INITIAL_SIZE: Size = Size::new(1000.0, 700.0);

/// The winit application: owns the window, the GPU-backed Vello renderer, the
/// swapchain surface, the embeddable [`UiRuntime`] frame driver, and the shared
/// [`ShowcaseState`] the interactive widgets drive.
struct ShowcaseWindow {
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
    /// live interaction (hover/focus/editing) state, and runs the real pipeline
    /// when a frame is due. The shared [`ShowcaseState`] lives behind an
    /// `Rc<RefCell<_>>` owned jointly by the command / text-input handlers (which
    /// mutate it) and the reactive view builder (which reads it back to rebuild
    /// [`showcase::view`] each frame) — both installed on this runtime, so the
    /// state needs no separate field here.
    runtime: UiRuntime,
    /// Last known cursor position in physical pixels, so a `MouseInput` (which
    /// carries no position) can be dispatched as a pointer event at the cursor.
    cursor: (f64, f64),
    /// The window's current HiDPI device-pixel ratio (winit `scale_factor`), so
    /// the app lays out in logical CSS pixels while rendering at the display's
    /// physical resolution. Updated from the window on `resumed` and on every
    /// `ScaleFactorChanged`.
    scale_factor: f64,
    /// The user zoom factor (Ctrl+= / Ctrl+- / Ctrl+0), multiplied with
    /// `scale_factor` to form the effective device scale. `1.0` is 100%.
    zoom: f32,
    /// The modifier keys currently held, tracked from winit `ModifiersChanged`
    /// so the keyboard handler can recognise the Ctrl+zoom chords before text is
    /// forwarded to the runtime's editable.
    modifiers: ModifiersState,
}

impl ShowcaseWindow {
    /// Builds the application with a fresh [`UiRuntime`] wired to the shared
    /// scene: the compiled stylesheet, the reactive view builder, and a handler
    /// per interactive command. No window, surface, or GPU device yet (created on
    /// `resumed`); image resources are also installed there since the sandbox dir
    /// is staged at startup.
    fn new(asset_dir: &std::path::Path) -> Self {
        let state = Rc::new(RefCell::new(ShowcaseState::default()));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), INITIAL_SIZE);

        // Real filesystem image loader + decoder, rooted at the staged sandbox so
        // `background-image: url("gallery.png")` and the `<img>` primitive resolve
        // to decoded pixels (same as `showcase_png.rs`).
        KitBootstrap::new().install_image_resources(&mut runtime, asset_dir);

        // Interactive wiring through the runtime's real command + reactive-render
        // path (no geometric hacks):
        //  - the Button's click resolves to CMD_BUTTON → bump the counter;
        //  - the Checkbox's change resolves to CMD_CHECKBOX → toggle checked;
        //  - the Input's per-keystroke edit fires CMD_INPUT with the resulting
        //    text → mirror it into `state.input_text`;
        //  - a view BUILDER rebuilds `showcase::view(&state)` each reactive frame;
        //  - the compiled showcase stylesheet is what the reactive path matches.
        {
            let s = state.clone();
            runtime.on_command(CMD_BUTTON, move || {
                let mut st = s.borrow_mut();
                st.counter += 1;
                println!("[button] counter = {}", st.counter);
            });
        }
        {
            let s = state.clone();
            runtime.on_command(CMD_CHECKBOX, move || {
                let mut st = s.borrow_mut();
                st.checked = !st.checked;
                println!("[checkbox] checked = {}", st.checked);
            });
        }
        {
            let s = state.clone();
            runtime.on_text_input(CMD_INPUT, move |text| {
                text.clone_into(&mut s.borrow_mut().input_text);
                println!("[input] value = {text:?}");
            });
        }
        {
            let s = state.clone();
            runtime.set_root_view(move || showcase::view(&s.borrow()));
        }
        runtime.set_stylesheets(vec![compile(showcase::stylesheet())]);

        Self {
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            zoom: 1.0,
            modifiers: ModifiersState::empty(),
        }
    }

    /// The effective device-pixel scale the runtime renders at: the window's
    /// HiDPI `scale_factor` times the user `zoom`. On a 2× display at 100% zoom
    /// this is `2.0`, so a logical 1000×700 window rasterizes into a 2000×1400
    /// physical swapchain — crisp, and no longer tiny.
    #[allow(clippy::cast_possible_truncation)]
    fn effective_scale(&self) -> f32 {
        (self.scale_factor as f32) * self.zoom
    }

    /// Assembles a domain button [`PointerEvent`] at the tracked cursor, reusing
    /// [`PointerConverter::kind`] to map the winit [`ElementState`] to the domain
    /// event kind (the converter exposes no builder for a button transition *at a
    /// position*, so the event is built directly — as in `windowed_counter.rs`).
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
    /// swapchain extent, HiDPI factor, and user zoom. Called after surface
    /// creation, on resize, on a scale-factor change, and after a zoom change.
    ///
    /// The viewport is the swapchain's **physical** extent divided by the
    /// effective scale, so layout runs in **logical** CSS pixels; the runtime's
    /// device scale is set to that same factor, so the composed scene is bracketed
    /// by a root `scale(s, s)` and rasterizes crisply at physical resolution. This
    /// is what keeps the UI from being tiny on a HiDPI display (and what the zoom
    /// shortcuts drive).
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

    /// Handles a Ctrl+zoom shortcut, returning `true` when `key` was a zoom chord
    /// (so the caller swallows it instead of forwarding it as text):
    ///
    /// - `Ctrl+=` / `Ctrl++` → zoom in (×1.1, capped at 5.0);
    /// - `Ctrl+-` → zoom out (÷1.1, floored at 0.3);
    /// - `Ctrl+0` → reset zoom to 1.0.
    ///
    /// A change re-syncs the viewport + device scale and requests a redraw, so the
    /// whole showcase grows/shrinks crisply about the new scale.
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

    /// Renders one frame: run a scheduler-gated reactive frame through the
    /// runtime (rebuilding the view from the current state when one is due), read
    /// the committed scene, package it, and present it on-screen via Vello.
    fn render_frame(&mut self) {
        // Run a REACTIVE frame only if one is due (§3.5); the runtime holds the
        // view builder + stylesheets, so it rebuilds `showcase::view` from the
        // current state itself. `None` means nothing was due — we still re-present
        // the last committed snapshot so a bare expose repaints rather than blanks.
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

        // Present on-screen (offscreen render + blit-to-surface, the documented
        // Vello 0.9 pattern the counter example uses).
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
                // A lost/outdated swapchain is recoverable: reconfigure, skip.
                if let (Some(surface), Some(device)) = (self.surface.as_ref(), self.device.as_ref())
                {
                    surface.configure(device);
                }
            }
        }
    }
}

impl ApplicationHandler for ShowcaseWindow {
    /// Creates the window, swapchain surface, and GPU-backed Vello renderer once
    /// the platform is ready, and wires the host text stack.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("obscur-kit — capability showcase (Vello)")
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
        // Let the platform IME deliver composition events for the editable Input.
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
                        // Seed the HiDPI factor from the freshly created window so
                        // the very first frame lays out in logical px and renders
                        // at physical resolution (no initial "tiny" frame).
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
                // Real domain path: winit → `PointerConverter::moved` →
                // `handle_pointer` → hit-test → hover / drag-selection.
                self.cursor = (position.x, position.y);
                let moved = PointerConverter::new().moved(PointerId::MOUSE, position);
                self.runtime.handle_pointer(moved);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                // Real domain pointer path for both transitions: `Down` anchors
                // the click and drives focus (seating the caret when it lands on
                // the Input), the matching `Up` completes it — `handle_pointer`
                // resolves a widget activation to CMD_BUTTON / CMD_CHECKBOX and
                // runs its handler, mutating the shared state.
                let event = self.pointer_button_event(state);
                let dispatch = self.runtime.handle_pointer(event);
                if let Some(command) = dispatch.command() {
                    println!("[click] resolved command {command:?}");
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                // Track the live modifier chord so the Ctrl+zoom shortcuts below
                // can be recognised (and swallowed) before text reaches the field.
                self.modifiers = modifiers.state();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // The window moved to a display with a different HiDPI ratio: keep
                // layout in logical px and re-scale. A `Resized` with the new
                // physical extent typically follows and re-syncs again.
                self.scale_factor = scale_factor;
                self.sync_viewport();
                self.runtime.request_frame(FrameReason::WindowChange);
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Ctrl+zoom shortcuts are handled here FIRST (on key-down only),
                // ahead of forwarding to the runtime, so a zoom chord adjusts the
                // effective device scale instead of being typed into the Input.
                if event.state == ElementState::Pressed
                    && self.modifiers.control_key()
                    && self.handle_zoom_shortcut(&event.logical_key)
                {
                    return;
                }
                // Real domain keyboard path: winit → `KeyboardConverter` →
                // `handle_key`. When the Input is focused, mutating keys drive its
                // editing session, whose `on_text_input` handler mirrors the text
                // into the shared state.
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        self.runtime.handle_key(KeyEvent::new(physical, logical, pressed));
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                // Real IME composition path onto the focused editable (spec §5.12).
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through the
/// real `obscur-css` frontend (identical to `showcase_png.rs`).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Writes the embedded showcase PNG into a fresh temp directory (the resource
/// loader's sandbox root) and returns it. A `url()` / `src` path resolves
/// relative to this directory.
fn stage_image_asset() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("obscur-kit-showcase-win-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create asset dir");
    std::fs::write(dir.join(showcase::IMAGE_ASSET_NAME), showcase::image_asset_bytes())
        .expect("write image asset");
    dir
}

/// Initialises a wgpu swapchain surface + device/queue for on-screen rendering
/// from an `Arc<Window>` (mirrors `windowed_counter.rs`).
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

/// Wires the full multi-font fallback stack onto the runtime and the Vello
/// renderer (identical to `windowed_counter.rs`): the primary under `sans-serif`
/// (id 0) plus a best-effort CJK fallback under the same family (id 1), so
/// resolution is by `cmap` COVERAGE, not name. The renderer receives the same
/// id↔bytes assignment so scene font handles line up with rasterized outlines.
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

/// The scene font handle (a `u32`) for a resolved [`FontId`]: the renderer and
/// the scene both key fonts by this value, so the two must agree.
fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}

/// Loads a system font's raw bytes for the text path, scanning well-known
/// Linux/macOS/Windows locations and returning the first that exists, or `None`.
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

/// Loads a second, CJK-covering fallback font's bytes (best effort), so a
/// mixed-script label has a covering font for ideographs the primary lacks.
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
    println!("== obscur-kit capability showcase → Vello (windowed, interactive) ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)\n");

    let asset_dir = stage_image_asset();
    let event_loop = EventLoop::new()?;
    let mut app = ShowcaseWindow::new(&asset_dir);
    event_loop.run_app(&mut app)?;
    Ok(())
}

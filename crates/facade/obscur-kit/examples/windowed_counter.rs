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
//! 1. **Window / event loop** — the window is created **through the toolkit's
//!    own §6 window port**, not by calling winit directly. The host issues
//!    [`WindowBackend::create_window`] on a
//!    [`WinitWindowBackend`] with domain [`WindowProperties`], and gets a
//!    [`WindowId`] back straight away; creation is *deferred*, so the OS window
//!    is materialized by the backend's [`WinitAppHandler`] on the next event-loop
//!    turn. This host therefore **wraps** that handler — every winit callback
//!    forwards to it first — and then reads what it left: the platform handle via
//!    [`WinitWindowBackend::native_window`] (an [`Arc<Window>`], which is what
//!    wgpu needs to build a `Surface<'static>`), and the window's lifecycle via
//!    the port's [`WindowBackend::poll_window_events`] — domain `Resized` drives
//!    the swapchain, domain `CloseRequested` exits.
//!
//!    **Input is deliberately not on that path.** The port carries §6.1 window
//!    lifecycle, not keystrokes: pointer/keyboard/IME stay on the raw winit
//!    events, converted by the adapter's `PointerConverter`/`KeyboardConverter`.
//!    That split is the design, not a gap.
//!
//!    **What this host still reaches around the port for** — stated because
//!    "created through the port" invites the reading that *everything* goes
//!    through it, and that is not true here:
//!
//!    - `window.request_redraw()` — frame scheduling is §3.5, not §6. The port
//!      does have a path (`UserEvent::WakeUp` makes the handler redraw every live
//!      window); this host calls the handle directly because it already holds it.
//!    - `window.set_ime_allowed(true)` — §5.12 IME, not a §6 window property.
//!    - `window.inner_size()` / `window.scale_factor()` at startup — the port
//!      reports later changes as domain `Resized` / `ScaleFactorChanged`, but has
//!      no pull-based accessor for the *initial* extent.
//!
//!    So the honest claim is narrower than "the host is backend-agnostic": window
//!    **creation, lifecycle and close** go through the port and name no winit
//!    type; presentation and input do not, and the `Arc<Window>` needed for the
//!    swapchain is fetched from the winit adapter, not the port. See
//!    `WinitWindowBackend::native_window` for why that handle is on the adapter.
//!
//!    Beware one unit trap when copying this host: domain `Resized` carries
//!    **physical** pixels while `WindowProperties::size` is **logical**. This
//!    example is correct only because both coincide at scale 1 — see the known
//!    defect noted on `window_winit::conversion::WindowEventConverter::convert`.
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
//! - `MouseInput` (left press *and* release) builds
//!   [`Down`](obscur_kit::interaction::pointer::PointerEventKind::Down) /
//!   [`Up`](obscur_kit::interaction::pointer::PointerEventKind::Up) events —
//!   assembled directly with [`PointerConverter::kind`] mapping the winit state,
//!   since the converter has no public builder for a button transition *at a
//!   position* — and dispatches both. The `Down` anchors the click; the `Up` on
//!   the same node completes it, and [`UiRuntime::handle_pointer`] resolves it to
//!   the hit button's command.
//!
//! The counter *value* is now driven through this **real command path** — no
//! geometric hack. Each [`Button`] binds its click to the reserved `EVENT_CLICK`
//! symbol carrying its own command (`CMD_INCREMENT` / `CMD_DECREMENT`); a
//! registered [`UiRuntime::on_command`] handler mutates a shared
//! `Rc<Cell<i32>>`, and the runtime's held view builder
//! ([`UiRuntime::set_root_view`]) rebuilds the counter from the new value on the
//! next reactive frame. So clicking the green bar increments the on-screen count
//! and the red bar decrements it, and hovering a bar lights its `:hover` style —
//! both through the same domain input path.
//!
//! # Text (multi-font fallback stack)
//!
//! The counter's labels ("Count 数字: N", "-", "+") are real text with a
//! deliberately **mixed-script** label. Fonts are a host resource (like the GPU
//! device): `resumed` loads a primary system font plus (best-effort) a second,
//! CJK-covering fallback font off disk and wires the FULL fallback stack via
//! [`wire_text_stack`]:
//!
//! - a [`UnicodeTextItemizer`] on the runtime ([`UiRuntime::set_text_itemizer`])
//!   splits each label into uniform bidi-level + script runs;
//! - a coverage-driven [`FontRegistry`] resolver
//!   ([`UiRuntime::set_font_resolver`]) picks, per run, the first registered font
//!   whose `cmap` covers it (both fonts registered under `sans-serif`, so
//!   COVERAGE — not name — disambiguates);
//! - a font-less [`HarfrustShaper`] ([`UiRuntime::set_text_shaper`]) shapes each
//!   item against its resolved face's in-memory bytes.
//!
//! The renderer is handed the **same** id->bytes assignment via
//! [`VelloRenderer::register_font`], so the scene's per-run font handle (the
//! resolved [`FontId`](obscur_kit::text::font::FontId)) selects the matching font
//! when Vello rasterizes. On a machine with both fonts present, the Latin part
//! draws from the primary and the CJK part from the fallback. With no fallback
//! found the CJK glyphs render as `.notdef` (the fallback path activates only once
//! a covering font is registered); with no font at all, text is skipped.
//!
//! # Environment note
//!
//! This example cannot run in the headless dev/CI environment (no display, no
//! GPU adapter), so it is only ever *compiled* there. On a real machine it opens
//! a window, presents the painted counter, and responds to clicks until closed.
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::command::KeyEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerId};
use obscur_kit::primitives::{Container, Text};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::renderer::RendererId;
use obscur_kit::render_vello::renderer::{VelloDevice, VelloRenderer, VelloSurface};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::font::{FontAttributes, FontFamily, GenericFamily};
use obscur_kit::text::ime::{CompositionEvent, Preedit};
use obscur_kit::text_harfrust::{FontRegistry, HarfrustShaper};
use obscur_kit::text_unicode::UnicodeTextItemizer;
use obscur_kit::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;
use obscur_kit::window_winit::backend::{UserEvent, WinitAppHandler, WinitWindowBackend};
use obscur_kit::window_winit::conversion::{KeyboardConverter, PointerConverter};
use obscur_kit::windowing::port::WindowBackend;
use obscur_kit::windowing::request::{CloseWindow, CreateWindow};
use obscur_kit::windowing::window::{WindowEvent as DomainWindowEvent, WindowProperties};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, Ime, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey as WinitPhysicalKey;
use winit::window::{Window, WindowId};

/// Commands the two buttons fire when activated (interned symbol handles).
const CMD_DECREMENT: InternedSymbol = InternedSymbol::from_raw(0x1001);
const CMD_INCREMENT: InternedSymbol = InternedSymbol::from_raw(0x1002);

/// The winit application: it owns the window, the GPU-backed renderer, the
/// swapchain surface, the embeddable [`UiRuntime`] frame driver, and the counter
/// UI state.
struct WindowedCounter {
    /// The toolkit's §6 window backend: the host issues `create_window` /
    /// `close_window` / `poll_window_events` on this, never on winit directly.
    backend: WinitWindowBackend,
    /// The backend's own event-loop handler, which this host wraps. Every winit
    /// callback forwards to it first; it materializes the deferred window,
    /// applies queued property work, and converts native events into the domain
    /// events `poll_window_events` hands back.
    handler: WinitAppHandler,
    /// The identity the port minted for our window. Valid from `create_window`
    /// onward — before the OS window behind it exists.
    window_id: obscur_kit::identity::WindowId,
    /// The live platform window, fetched from the backend via `native_window`
    /// once the handler has materialized it (an [`Arc`] so wgpu can build a
    /// `Surface<'static>` from it).
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
    /// frame is due. Pointer input is dispatched into it via `handle_pointer`,
    /// which resolves a click on a button to its bound command and runs the
    /// registered handler; the reactive builder then rebuilds the view.
    runtime: UiRuntime,
    /// The counter value, shared between the click handlers (which mutate it) and
    /// the reactive view builder (which reads it) — the canonical `Rc<Cell<i32>>`
    /// bridge. Registered handlers for `CMD_INCREMENT`/`CMD_DECREMENT` bump it,
    /// and the [`UiRuntime::set_root_view`] builder rebuilds the counter view from
    /// its current value each reactive frame.
    count: Rc<Cell<i32>>,
    /// Last known cursor position in physical pixels, tracked so a `MouseInput`
    /// (which carries no position) can be dispatched as a pointer event at the
    /// cursor.
    cursor: (f64, f64),
}

impl WindowedCounter {
    /// Builds the application with a fresh [`UiRuntime`] and the two-button
    /// counter UI, but no window, surface, or GPU device yet (those are created
    /// on `resumed`). The runtime starts at a placeholder viewport matching the
    /// initial window size; [`sync_viewport`](Self::sync_viewport) sets the real
    /// physical extent once the surface exists.
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let count = Rc::new(Cell::new(0));
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(480.0, 240.0));

        // Wire the interactive counter through the runtime's real command +
        // reactive-render path (no geometric hack):
        //  - register a handler per button command that mutates the shared count;
        //  - install a view BUILDER that rebuilds the counter view from the
        //    current count each reactive frame;
        //  - install the stylesheet the reactive path matches against.
        // A click on a button then resolves (via the reserved `EVENT_CLICK`
        // symbol the button binds) to its command, runs the handler, and the next
        // reactive frame rebuilds the view with the new value.
        {
            let c = count.clone();
            runtime.on_command(CMD_INCREMENT, move || c.set(c.get() + 1));
        }
        {
            let c = count.clone();
            runtime.on_command(CMD_DECREMENT, move || c.set(c.get() - 1));
        }
        {
            let c = count.clone();
            runtime.set_root_view(move || build_counter_view(c.get()));
        }
        runtime.set_stylesheets(vec![counter_stylesheet()]);

        // Create the window THROUGH THE PORT: domain properties in, a `WindowId`
        // out, no winit type named. The id is live immediately; the OS window it
        // names is built by `handler` on the first event-loop turn (deferred
        // creation), which is why `window`/`surface` stay `None` until `resumed`.
        let mut backend = WinitWindowBackend::new();
        let window_id = backend.create_window(CreateWindow::new(
            WindowProperties::default()
                .with_title("obscur-kit — windowed counter")
                .with_size(Size::new(480.0, 240.0)),
        ))?;
        let handler = backend.app_handler();

        Ok(Self {
            backend,
            handler,
            window_id,
            window: None,
            surface: None,
            device: None,
            renderer: None,
            runtime,
            count,
            cursor: (0.0, 0.0),
        })
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

    /// Drains the **port's** domain window events and acts on them (spec §6.1).
    ///
    /// This is the §6.1 lifecycle path: the backend's handler converted the native
    /// events, and the host reads them as backend-agnostic
    /// [`DomainWindowEvent`]s — the same values a WASM or headless backend would
    /// report — rather than matching on winit variants.
    ///
    /// - `Resized` reconfigures the swapchain and the layout viewport.
    /// - `CloseRequested` is a *request*: §6.1 leaves the decision to the
    ///   application, so the host answers it by asking the port to close the
    ///   window and then leaving the loop.
    fn drain_window_events(&mut self, event_loop: &ActiveEventLoop) {
        for (id, event) in self.backend.poll_window_events() {
            match event {
                DomainWindowEvent::Resized(size) => {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let (width, height) = (size.width as u32, size.height as u32);
                    if let (Some(surface), Some(device)) =
                        (self.surface.as_mut(), self.device.as_ref())
                    {
                        surface.resize(device, width, height);
                    }
                    // Keep the layout viewport in step with the new extent (the
                    // surface/device borrows above have ended) and schedule a
                    // frame so the next redraw re-lays-out under it.
                    self.sync_viewport();
                    self.runtime.request_frame(FrameReason::WindowChange);
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
                DomainWindowEvent::CloseRequested => {
                    // We honor it: close through the port, then leave.
                    let result = self.backend.close_window(CloseWindow::request(id));
                    println!("[window] close requested -> port answered {result:?}");
                    event_loop.exit();
                }
                _ => {}
            }
        }
    }

    /// Renders one frame: rebuild the view, run a scheduler-gated frame through
    /// the runtime, read the committed scene, package it, and present it
    /// on-screen via the Vello surface path.
    fn render_frame(&mut self) {
        // --- Stage 1+2: run a REACTIVE frame only if one is due (§3.5). -------
        // The runtime holds the view BUILDER (installed in `new`) and its
        // stylesheets, so it rebuilds the counter view from the current `count`
        // itself — no `&View` is threaded here. First paint, input (hover/focus),
        // resize, and each click-dispatched command `request_frame` a reason;
        // `run_due_frame_reactive` produces at most one frame per redraw and
        // returns `None` when nothing is pending. We still present the (unchanged)
        // snapshot on a `None` so a bare `RedrawRequested` — e.g. an OS expose —
        // repaints the last committed scene rather than a blank.
        let produced = self.runtime.run_due_frame_reactive();
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

impl ApplicationHandler<UserEvent> for WindowedCounter {
    /// Picks up the window the **port** created, then builds the swapchain
    /// surface and the GPU-backed renderer on it.
    ///
    /// The window itself is not created here — `WindowedCounter::new` already
    /// asked the port for it. What happens here is the other half of deferred
    /// creation: forwarding to `self.handler` is what materializes the queued
    /// OS window, and `native_window` is what hands this host the `Arc<Window>`
    /// wgpu needs. Forward first, read second; the reverse order sees `None`.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Let the backend's handler service the loop turn: it builds the window
        // queued by `create_window` and enumerates the displays.
        self.handler.resumed(event_loop);

        // `resumed` can fire more than once (e.g. on Android); build only once.
        if self.window.is_some() {
            return;
        }

        // 1. Take the live platform handle for the id the port minted for us.
        //    `None` here means the backend could not materialize the window;
        //    it records the reason rather than panicking.
        let Some(window) = self.backend.native_window(self.window_id) else {
            eprintln!("the window backend did not materialize a window");
            event_loop.exit();
            return;
        };
        // Allow the platform IME to deliver composition events for this window
        // (spec §5.12). A real app toggles this per focused editable; the counter
        // has no text field, so it is enabled once here — the `WindowEvent::Ime`
        // arm below then drives `UiRuntime::handle_ime` end-to-end. IME is not a
        // §6 window property, so it is set on the platform handle directly.
        window.set_ime_allowed(true);

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
                        // Text is a host resource (like the GPU device). Wire the
                        // FULL multi-font fallback stack: the runtime itemizes
                        // each label into uniform bidi+script runs
                        // (`UnicodeTextItemizer`), resolves each run to the first
                        // registered font whose `cmap` COVERS it (`FontRegistry`),
                        // and shapes it against that resolved face's in-memory
                        // bytes (font-less `HarfrustShaper`). The renderer gets the
                        // SAME id->bytes assignment so scene font handles line up
                        // with the fonts it rasterizes.
                        if let Some(primary_bytes) = load_system_font() {
                            wire_text_stack(&mut self.runtime, &mut renderer, primary_bytes);
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

    /// Routes window events.
    ///
    /// Every native event is forwarded to the backend's handler first, which
    /// converts the §6.1 lifecycle ones (resize, close, focus, scale) into domain
    /// [`DomainWindowEvent`]s for [`Self::drain_window_events`] to act on. What is
    /// handled *here* is what the window port deliberately does not carry: pointer,
    /// keyboard and IME input, plus the redraw that presents a frame.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Forward first: this is what buffers the domain lifecycle events that
        // `drain_window_events` reads on the next `about_to_wait`.
        self.handler
            .window_event(event_loop, window_id, event.clone());

        match event {
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
            WindowEvent::MouseInput { state, button, .. } if button == MouseButton::Left => {
                // Feed the *real* domain pointer path for BOTH transitions: a
                // `Down` at the cursor anchors the click and drives focus; the
                // matching `Up` on the same node completes the click, and
                // `handle_pointer` resolves it (via the button's reserved
                // `EVENT_CLICK` binding) to `CMD_INCREMENT`/`CMD_DECREMENT` and
                // runs the registered handler, which mutates the shared count.
                // The dispatch itself requests a frame; the next redraw's
                // `run_due_frame_reactive` rebuilds the view with the new value.
                let event = self.pointer_button_event(state);
                let dispatch = self.runtime.handle_pointer(event);
                if let Some(command) = dispatch.command() {
                    println!(
                        "[click] resolved command {command:?} -> count = {}",
                        self.count.get()
                    );
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Feed the *real* domain keyboard path, mirroring `CursorMoved`
                // above: winit → `KeyboardConverter` → `handle_key` → focused
                // node → command resolution. The converter splits the winit
                // `KeyEvent` into its layout-independent physical key and its
                // layout-dependent logical key; only a modeled physical position
                // produces a domain event (media/IME keys convert to `None`).
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        let dispatch = self
                            .runtime
                            .handle_key(KeyEvent::new(physical, logical, pressed));
                        if let Some(command) = dispatch.command() {
                            println!("[key] resolved command: {command:?}");
                        }
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::Ime(ime) => {
                // Feed the *real* IME composition path: winit's `Ime` events map
                // 1:1 onto the domain `CompositionEvent`s the runtime consumes
                // (spec §5.12). `Enabled` starts a composition, `Preedit` carries
                // the uncommitted (shown-but-not-inserted) preedit, `Commit`
                // finalizes the composed text into the focused editable as one
                // edit, and `Disabled` cancels. `handle_ime` is a no-op when no
                // editable is focused. The winit preedit cursor is a byte range
                // within the preedit; its start seats the display caret.
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
            // Close is NOT handled here: the handler converted it into a domain
            // `CloseRequested`, and `drain_window_events` decides what to do with
            // it — which is what §6.1 means by "the application decides whether to
            // honor it".
            _ => {}
        }
    }

    /// Services the backend's queued work each turn, then acts on the domain
    /// window events it produced.
    ///
    /// Forwarding to the handler here is not optional: it is what applies any
    /// operation the host issued on the backend between turns (the
    /// `close_window` below, property updates, later `create_window` calls).
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handler.about_to_wait(event_loop);
        self.drain_window_events(event_loop);
    }

    /// Forwards the frame scheduler's cross-thread wake-up to the backend, which
    /// turns it into a redraw request on every live window (spec §3.5).
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        self.handler.user_event(event_loop, event);
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

/// Loads a *second*, different-script fallback font's bytes, for the multi-font
/// fallback demo.
///
/// The primary system font ([`load_system_font`]) typically covers Latin (and
/// often Cyrillic); this scans a few well-known CJK faces so a mixed-script label
/// has a covering font for the ideographs the primary lacks. Returns `None` when
/// none are present — the example still compiles and runs with only the primary
/// (the mixed-script glyphs then render as `.notdef`; see [`wire_text_stack`]).
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

/// Wires the full multi-font fallback stack onto the runtime and the renderer.
///
/// Builds a [`FontRegistry`] (the coverage-driven [`FontResolver`]) and registers
/// the primary under the `sans-serif` generic family — the family the runtime's
/// default fallback chain (`FontFallback::default_chain`) consults — so it gets
/// id `0`. If a fallback font is found it is registered under the **same** family
/// (id `1`), so resolution picks between the two by `cmap` COVERAGE, not by name:
/// a Latin run resolves to whichever covers it, a CJK run to the CJK face.
///
/// The renderer receives the **same** id->bytes assignment via
/// [`VelloRenderer::register_font`], so the scene's per-run font handles (the
/// resolved [`FontId`]s) line up with the fonts the renderer rasterizes.
///
/// Without a fallback font found, mixed-script glyphs the primary lacks resolve to
/// no covering font and render as `.notdef`; the fallback path activates only once
/// a covering font is registered here.
fn wire_text_stack(runtime: &mut UiRuntime, renderer: &mut VelloRenderer, primary_bytes: Vec<u8>) {
    let mut registry = FontRegistry::new();

    // Primary under the SansSerif generic family -> id 0.
    let primary_id = registry.register(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        primary_bytes.clone(),
    );
    renderer.register_font(font_handle(primary_id), primary_bytes);

    // Fallback under the SAME family -> id 1 (coverage, not name, disambiguates).
    if let Some(fallback_bytes) = load_fallback_font() {
        let fallback_id = registry.register(
            FontFamily::generic(GenericFamily::SansSerif),
            FontAttributes::default(),
            fallback_bytes.clone(),
        );
        renderer.register_font(font_handle(fallback_id), fallback_bytes);
    } else {
        // No second font: the primary alone can't cover the CJK part of the
        // demo label, so those glyphs paint as .notdef. Registering a covering
        // face above is what activates the fallback path.
        eprintln!(
            "no fallback font found; mixed-script glyphs the primary lacks render as .notdef"
        );
    }

    // Install the stack: resolve-by-coverage + itemize + per-item shaping.
    // The shaper answers line-metric queries by `FontId` alone, so it needs the
    // registry's own per-face metrics; without them the glyphs would be real but
    // every line box sized from idealized ratios.
    let shaper = HarfrustShaper::without_font().with_faces(registry.face_metrics());
    runtime.set_font_resolver(Box::new(registry));
    runtime.set_text_itemizer(Box::new(UnicodeTextItemizer::new()));
    runtime.set_text_shaper(Box::new(shaper));
}

/// The scene font handle (a `u32`) for a resolved [`FontId`]: the renderer and
/// the scene both key fonts by this value, so the two must agree.
fn font_handle(id: obscur_kit::text::font::FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
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

/// Builds the counter UI from `count`: a dark `.counter` panel stacking a label
/// bar showing the value and the two real [`Button`]s, each rendered as a bar
/// that decrements/increments on click.
///
/// This is the reactive view BUILDER the runtime holds (installed in `new` via
/// [`UiRuntime::set_root_view`]); the runtime re-invokes it each reactive frame,
/// so passing the current `count` rebuilds the label after a click bumps it.
fn build_counter_view(count: i32) -> View {
    // Mixed-script label (Latin + CJK): on a machine with both a Latin primary
    // and a CJK fallback registered, each script renders from its covering font
    // via the coverage resolver; with only the primary, the CJK part is .notdef.
    let label = tagged(
        "bar label",
        vec![Text::new(format!("Count 数字: {count}")).into_view_node()],
    );
    let minus = button_bar("bar decrement", "-", CMD_DECREMENT);
    let plus = button_bar("bar increment", "+", CMD_INCREMENT);
    let container = Container::new()
        .child(label)
        .child(minus)
        .child(plus)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into())));
    View::single(container)
}

/// Builds a real [`Button`] bound to `command` and rendered as a sizing bar.
///
/// The sizing `class` is placed on the **button's own element node** (not a
/// wrapping container) for a reason: the runtime resolves a click by bubbling
/// from the hit node up its *ancestors* to the first `EVENT_CLICK` binding. If
/// the button were wrapped in the sized container, the pointer would hit the
/// container and the button (a descendant) would never be on that ancestor
/// chain, so the click would resolve to nothing. Sizing the button node itself
/// makes it the hit target, so its own `EVENT_CLICK → command` binding fires.
fn button_bar(class: &str, label: &str, command: InternedSymbol) -> ViewNode {
    Button::new(label, command)
        .view()
        .into_fragment()
        .nodes()
        .to_vec()
        .remove(0)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
}

/// Builds the winit event loop and runs the application to completion.
///
/// # Errors
///
/// Propagates any event-loop construction or run error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== obscur-kit windowed counter ==");
    println!("(full backend: winit window + wgpu/Vello GPU renderer, on-screen present)\n");

    // A `UserEvent` loop, because the backend's handler is an
    // `ApplicationHandler<UserEvent>`: that is the §3.5 cross-thread wake-up
    // channel a frame scheduler nudges the loop through.
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = WindowedCounter::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}

#![doc = include_str!("runner.doc/_module.md")]

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Ime, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::PhysicalKey as WinitPhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

use super::AppError;
use super::app::FrameInfo;
use super::app_parts::AppParts;
use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::control::AppControl;
use crate::geometry::{Point, Size};
use crate::interaction::command::KeyEvent;
use crate::interaction::pointer::{PointerButton, PointerButtons, PointerEvent, PointerId};
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{
    AcquireWake, SurfaceAcquirer, VelloDevice, VelloRenderer, VelloSurface,
};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::text::ime::{CompositionEvent, Preedit};
use crate::view::view::View;
use crate::window_winit::conversion::{KeyboardConverter, PointerConverter};

#[doc = include_str!("runner.doc/MAX_FRAME_DELTA.md")]
const MAX_FRAME_DELTA: f32 = 0.1;

#[doc = include_str!("runner.doc/FrameStats.md")]
#[derive(Default)]
struct FrameStats {
    frames: u32,
    build: std::time::Duration,
    snapshot: std::time::Duration,
    present: std::time::Duration,
    worst: std::time::Duration,
}

#[doc = include_str!("runner.doc/REPORT_EVERY.md")]
const REPORT_EVERY: u32 = 60;

impl FrameStats {
    #[doc = include_str!("runner.doc/record.md")]
    fn record(
        &mut self,
        build: std::time::Duration,
        snapshot: std::time::Duration,
        present: std::time::Duration,
    ) {
        self.frames += 1;
        self.build += build;
        self.snapshot += snapshot;
        self.present += present;
        self.worst = self.worst.max(build + snapshot + present);
        if self.frames < REPORT_EVERY {
            return;
        }
        let n = f64::from(self.frames);
        let ms = |total: std::time::Duration| total.as_secs_f64() * 1000.0 / n;
        let total = ms(self.build) + ms(self.snapshot) + ms(self.present);
        println!(
            "[frame-stats] {n:.0} frames | build {:.2}ms | snapshot {:.2}ms | present {:.2}ms | \
             total {total:.2}ms (~{:.0} fps if saturated) | worst {:.2}ms",
            ms(self.build),
            ms(self.snapshot),
            ms(self.present),
            1000.0 / total.max(0.001),
            self.worst.as_secs_f64() * 1000.0,
        );
        *self = Self::default();
    }
}

#[doc = include_str!("runner.doc/run.md")]
pub(super) fn run(
    parts: AppParts,
    builder: Box<dyn FnMut() -> View + 'static>,
) -> Result<(), AppError> {
    run_inner(parts, builder, None)
}

#[doc = include_str!("runner.doc/run_with_frame.md")]
pub(super) fn run_with_frame(
    parts: AppParts,
    mut builder: Box<dyn FnMut(&FrameInfo) -> View + 'static>,
) -> Result<(), AppError> {
    let info = Rc::new(Cell::new(FrameInfo {
        viewport: Size::new(parts.size.0, parts.size.1),
        scale: 1.0,
    }));
    let seen = Rc::clone(&info);
    let wrapped: Box<dyn FnMut() -> View + 'static> = Box::new(move || builder(&seen.get()));
    run_inner(parts, wrapped, Some(info))
}

fn run_inner(
    parts: AppParts,
    builder: Box<dyn FnMut() -> View + 'static>,
    frame_info: Option<Rc<Cell<FrameInfo>>>,
) -> Result<(), AppError> {
    // Read before `boot` consumes the parts. A layer surface has no title and no
    // requested size — the platform decides both — so these belong to the window
    // runner rather than to the shared `BootedApp`, where they would sit unread in
    // a build with no windowing backend.
    let (title, app_id, size) = (parts.title.clone(), parts.app_id.clone(), parts.size);
    let (width, height) = size;
    let booted = parts.boot(builder, Size::new(width, height));

    // Built before the runner: the acquire thread needs something to wake this
    // loop with, and `Window::request_redraw` is not it — it marks the window
    // dirty without promising to rouse a loop parked in `wait`, which is exactly
    // where a frame skipped for want of a swapchain image leaves it.
    let event_loop = EventLoop::new().map_err(|error| AppError::EventLoop(error.to_string()))?;
    let proxy = event_loop.create_proxy();

    let mut app = AppRunner {
        title,
        app_id,
        size,
        runtime: booted.runtime,
        css: booted.css,
        control: booted.control,
        window: None,
        acquirer: None,
        extent: (0, 0),
        proxy: Some(proxy),
        renderer: None,
        scale_factor: 1.0,
        cursor: (0.0, 0.0),
        held_buttons: PointerButtons::new(),
        // The cursor is outside the window until winit says otherwise, so the
        // first move it reports is an entry.
        entry_pending: true,
        epoch: std::time::Instant::now(),
        last_frame: None,
        init_failure: None,
        scroll: booted.scroll,
        ime_allowed: false,
        frame_info,
        stats: std::env::var_os("OBSCUR_FRAME_STATS").map(|_| FrameStats::default()),
        fonts: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|error| AppError::EventLoop(error.to_string()))?;

    // A GPU/window failure exits the loop cleanly on `resumed`, recording the
    // reason so `run` can report it rather than returning `Ok` on a blank window.
    match app.init_failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

#[doc = include_str!("runner.doc/install_text.md")]
#[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
fn install_text(
    boot: &KitBootstrap,
    runtime: &mut UiRuntime,
    renderer: &mut VelloRenderer,
) -> crate::bootstrap::InstalledFonts {
    // The application's own faces and the host's generics, in that order — a
    // named family an application shipped is only a font from here on.
    let font_set = boot.font_set();

    let installed = boot.install_text_stack(runtime, &font_set);
    for (handle, bytes) in installed.iter() {
        renderer.register_font(handle, bytes.to_vec());
    }
    installed
}

#[cfg(not(all(feature = "text-harfrust", feature = "text-unicode")))]
fn install_text(
    _boot: &KitBootstrap,
    _runtime: &mut UiRuntime,
    _renderer: &mut VelloRenderer,
) -> crate::bootstrap::InstalledFonts {
    crate::bootstrap::InstalledFonts::default()
}

#[doc = include_str!("runner.doc/AppRunner.md")]
struct AppRunner {
    title: String,
    app_id: String,
    size: (f32, f32),
    runtime: UiRuntime,
    #[doc = include_str!("runner.doc/css.md")]
    css: CssPipeline,
    #[doc = include_str!("runner.doc/control.md")]
    control: AppControl,
    window: Option<Arc<Window>>,
    #[doc = include_str!("runner.doc/acquirer.md")]
    acquirer: Option<SurfaceAcquirer>,
    #[doc = include_str!("runner.doc/extent.md")]
    extent: (u32, u32),
    #[doc = include_str!("runner.doc/proxy.md")]
    proxy: Option<EventLoopProxy<()>>,
    renderer: Option<VelloRenderer>,
    #[doc = include_str!("runner.doc/scale_factor.md")]
    scale_factor: f64,
    #[doc = include_str!("runner.doc/cursor.md")]
    cursor: (f64, f64),
    #[doc = include_str!("runner.doc/held_buttons.md")]
    held_buttons: PointerButtons,
    #[doc = include_str!("runner.doc/entry_pending.md")]
    entry_pending: bool,
    #[doc = include_str!("runner.doc/epoch.md")]
    epoch: std::time::Instant,
    #[doc = include_str!("runner.doc/last_frame.md")]
    last_frame: Option<std::time::Instant>,
    #[doc = include_str!("runner.doc/init_failure.md")]
    init_failure: Option<AppError>,
    #[doc = include_str!("runner.doc/scroll.md")]
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    #[doc = include_str!("runner.doc/ime_allowed.md")]
    ime_allowed: bool,
    #[doc = include_str!("runner.doc/frame_info.md")]
    frame_info: Option<Rc<Cell<FrameInfo>>>,
    #[doc = include_str!("runner.doc/stats.md")]
    stats: Option<FrameStats>,
    #[doc = include_str!("runner.doc/fonts.md")]
    fonts: Option<crate::bootstrap::InstalledFonts>,
}

impl AppRunner {
    #[doc = include_str!("runner.doc/now_micros.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn now_micros(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64
    }

    #[doc = include_str!("runner.doc/scale.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    #[doc = include_str!("runner.doc/sync_surface.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn sync_surface(&mut self) {
        // The extent is tracked here rather than read off the surface: the
        // surface lives on the acquire thread now, and asking it would mean
        // reaching across the very boundary that keeps this thread unblocked.
        let (width, height) = self.extent;
        if width == 0 || height == 0 {
            return;
        }
        self.runtime
            .set_surface(Size::new(width as f32, height as f32), self.scale());
        if let Some(info) = self.frame_info.as_ref() {
            let scale = self.scale();
            info.set(FrameInfo {
                // Logical, because that is the space the app's CSS and its
                // layout are in; the physical extent is the runner's business.
                viewport: Size::new(width as f32 / scale, height as f32 / scale),
                scale,
            });
        }
    }

    #[doc = include_str!("runner.doc/render_frame.md")]
    #[allow(clippy::cast_precision_loss)]
    fn render_frame(&mut self) {
        // Only present when a frame was actually produced. A redraw request that
        // finds nothing due — a bare cursor move over unchanged UI, which winit
        // delivers continuously — would otherwise rebuild the view, re-snapshot and
        // re-present an identical scene on every pointer twitch. The runtime is the
        // authority on whether anything changed: it schedules a frame itself when
        // hover, focus, animation or a landed resource makes one necessary.
        // Time passes here or nowhere. The interaction and motion domains take
        // the clock as data, so a host must supply it, and this one never did:
        // `advance_motion` had no caller in any windowed path, which left every
        // CSS transition and `@keyframes` animation frozen in a real window while
        // passing every headless test (the test harnesses tick it themselves).
        // §4.17's "active animations keep requesting frames" was half-true — the
        // runtime asked for the frames, and nothing moved the clock they were for.
        // The swapchain image BEFORE anything else. Acquiring it blocks until the
        // compositor releases a buffer, and this is the thread winit delivers
        // input on — so acquiring here would stop the application hearing the
        // pointer for as long as the wait lasts (audit 1-D, and core principle 5:
        // platform calls are to be asynchronous and synchronous work
        // frame-bounded). A `SurfaceAcquirer` does the waiting elsewhere.
        //
        // No image yet means no frame this turn. The pending frame stays pending
        // because nothing below has run, and the acquire thread asks winit for a
        // redraw when one lands — so nothing spins and nothing is lost. §3.5
        // permits the skip in so many words.
        let image = if self.runtime.has_pending_frame() {
            let Some(image) = self.acquirer.as_mut().and_then(SurfaceAcquirer::try_take) else {
                if let Some(acquirer) = self.acquirer.as_mut() {
                    acquirer.request();
                }
                return;
            };
            // The extent it was taken at against the extent the window has now.
            // A blit scales whatever it is handed, so a stale image is a frame at
            // the wrong size and nothing downstream can tell — the layer path met
            // exactly that as audit 1-E. One dropped frame, then a resize, and the
            // pending frame is still pending because it has not been run.
            let (width, height) = self.extent;
            if !image.matches(width, height) {
                drop(image);
                if let Some(acquirer) = self.acquirer.as_mut() {
                    acquirer.resize(width, height);
                }
                return;
            }
            image
        } else {
            return;
        };

        let started = std::time::Instant::now();
        let delta = self
            .last_frame
            .replace(started)
            // Clamped: a window that was occluded, dragged between monitors or
            // simply starved must not resume by jumping an animation forward by
            // however many seconds the gap was.
            .map_or(0.0, |prev| {
                (started - prev).as_secs_f32().min(MAX_FRAME_DELTA)
            });
        if delta > 0.0 {
            self.runtime.advance_motion(delta);
        }
        // A request queued since the last frame — a replaced stylesheet, a queued
        // runtime action — reaches the frame that is about to be built rather than the
        // one after it. Guarded because the steady state is "nothing queued", and this
        // runs on every redraw.
        if self.control.has_pending() {
            self.control.apply(&mut self.runtime, &mut self.css);
        }
        if self.runtime.run_due_frame_reactive().is_none() {
            // Nothing was due, but the tick above may have scheduled the *next*
            // animation frame; without asking for it the loop would sleep and the
            // animation would stall on its first idle turn.
            if self.runtime.has_pending_frame() {
                self.request_redraw();
            }
            return;
        }
        let built = started.elapsed();
        // The one point in the frame where application code may touch the runtime
        // (spec §2.6, §4.12): the frame has committed, so the tree exists and a
        // `test-id` can resolve to a real node, and the next frame has not started, so
        // nothing is mid-propagation. A no-op with nothing queued and every name
        // already bound, which is the steady state.
        self.control.apply(&mut self.runtime, &mut self.css);
        let snapshot_at = std::time::Instant::now();
        let snapshot = self.runtime.snapshot();
        let snapshotted = snapshot_at.elapsed();
        let present_at = std::time::Instant::now();

        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        // A character outside every registered face pulls one in from the host
        // while the frame is being built. The renderer rasterizes from its own
        // id→bytes table, so it has to be told before this frame is presented —
        // otherwise the glyph resolves, the run shapes, and nothing is drawn.
        // Empty on every frame where no new character appeared, which is nearly
        // all of them.
        if let Some(fonts) = self.fonts.as_ref() {
            for (handle, bytes) in fonts.take_discovered() {
                renderer.register_font(handle, bytes.to_vec());
            }
        }

        // The extent came with the image: the surface is on the acquire thread,
        // and an image acquired before a resize must not be blitted against the
        // new numbers.
        let physical = Size::new(image.width as f32, image.height as f32);
        let submission = FrameSubmission::new(RenderFrame::new(snapshot, physical));
        if let Err(error) = renderer.present_to_texture(&submission, image) {
            // A lost surface is reconfigured by the acquire thread on its next
            // attempt; there is nothing to reconfigure from here.
            eprintln!("present failed: {error:?}");
        }
        if let Some(stats) = self.stats.as_mut() {
            stats.record(built, snapshotted, present_at.elapsed());
        }
        // While motion (or a fling) remains, keep the loop turning. The runtime is
        // the authority on whether another frame is owed, so an idle UI still
        // sleeps — this is not a render-forever loop.
        if self.runtime.has_pending_frame() {
            self.request_redraw();
        }
    }

    #[doc = include_str!("runner.doc/request_redraw.md")]
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    #[doc = include_str!("runner.doc/sync_ime.md")]
    fn sync_ime(&mut self) {
        let wanted = self.runtime.focus_accepts_text();
        if wanted == self.ime_allowed {
            return;
        }
        self.ime_allowed = wanted;
        if let Some(window) = self.window.as_ref() {
            window.set_ime_allowed(wanted);
        }
    }

    #[doc = include_str!("runner.doc/pointer_button_event.md")]
    #[allow(clippy::cast_possible_truncation)]
    #[doc = include_str!("runner.doc/pointer_button_event_2.md")]
    fn pointer_button_event(&mut self, button: PointerButton, state: ElementState) -> PointerEvent {
        let at = PhysicalPosition::new(self.cursor.0, self.cursor.1);
        let converter = PointerConverter::new();
        self.held_buttons = held_after(self.held_buttons, button, state);
        match state {
            ElementState::Pressed => {
                converter.pressed(PointerId::MOUSE, at, button, self.held_buttons)
            }
            ElementState::Released => {
                converter.released(PointerId::MOUSE, at, button, self.held_buttons)
            }
        }
        .with_timestamp(self.now_micros())
    }
}

impl ApplicationHandler for AppRunner {
    #[doc = include_str!("runner.doc/resumed.md")]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(unused_mut))]
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone())
            .with_transparent(true)
            .with_inner_size(LogicalSize::new(
                f64::from(self.size.0),
                f64::from(self.size.1),
            ));
        // The Wayland `app_id` / X11 `WM_CLASS` — the stable identity a compositor
        // matches window rules on (Hyprland's `class`, sway's `app_id`) and the one
        // a taskbar keys off. winit exposes it only through the per-backend
        // extension traits, and setting both is correct: they write different
        // platform fields, so an X11 session and a Wayland one each get the id.
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            {
                use winit::platform::wayland::WindowAttributesExtWayland as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
            {
                use winit::platform::x11::WindowAttributesExtX11 as _;
                attributes = attributes.with_name(self.app_id.clone(), String::new());
            }
        }
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.init_failure = Some(AppError::Window(error.to_string()));
                event_loop.exit();
                return;
            }
        };
        // Text entry starts OFF and is offered per focus by `sync_ime` below. It
        // used to be switched on here, once, for the window's whole life — which
        // reads as "IME composition works out of the box" on a desktop and as "the
        // on-screen keyboard covers half the tablet" on a tablet, in an app with
        // no field to type into.
        window.set_ime_allowed(false);

        let (surface, config, device, queue, gpu) = match init_gpu(&window) {
            Ok(gpu) => gpu,
            Err(error) => {
                self.init_failure = Some(AppError::Gpu(error));
                event_loop.exit();
                return;
            }
        };
        self.runtime.report_diagnostic(&gpu_diagnostic(&gpu));
        let vello_surface = VelloSurface::new(surface, config);
        vello_surface.configure(&device);
        let vello_device = match VelloDevice::new(device.clone(), queue) {
            Ok(vello_device) => vello_device,
            Err(error) => {
                self.init_failure = Some(AppError::Gpu(error.to_string()));
                event_loop.exit();
                return;
            }
        };
        let mut renderer = VelloRenderer::new(RendererId::from_raw(1)).with_device(vello_device);
        self.fonts = Some(install_text(
            &KitBootstrap::new(),
            &mut self.runtime,
            &mut renderer,
        ));

        self.scale_factor = window.scale_factor();
        self.extent = (vello_surface.config().width, vello_surface.config().height);

        // The surface moves onto the acquire thread, and its wake is an
        // `EventLoopProxy`: a user event both rouses a parked loop and lands on
        // this thread, which `request_redraw` alone does not promise.
        let wake: Option<AcquireWake> = self.proxy.clone().map(|proxy| {
            Arc::new(move || {
                let _ = proxy.send_event(());
            }) as AcquireWake
        });
        self.acquirer = Some(SurfaceAcquirer::spawn(vello_surface, device, wake));

        self.renderer = Some(renderer);
        self.window = Some(window);
        self.sync_surface();
        self.runtime.request_frame(FrameReason::UiCommit);
        self.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                self.extent = (new_size.width, new_size.height);
                if let Some(acquirer) = self.acquirer.as_mut() {
                    // Also discards anything already acquired: an image fetched
                    // at the old extent would present stretched.
                    acquirer.resize(new_size.width, new_size.height);
                }
                self.sync_surface();
                self.runtime.request_frame(FrameReason::WindowChange);
                self.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.sync_surface();
                self.runtime.request_frame(FrameReason::WindowChange);
                self.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                let converter = PointerConverter::new();
                // The entry winit could not place: `CursorEntered` carries no
                // position, and this is the first event that knows one. See
                // `entry_pending` — synthesising the crossing back there would
                // enter whatever node sat where the pointer *left*.
                let event = if std::mem::take(&mut self.entry_pending) {
                    converter.entered(PointerId::MOUSE, position, self.held_buttons)
                } else {
                    converter.moved(PointerId::MOUSE, position, self.held_buttons)
                };
                self.runtime
                    .handle_pointer(event.with_timestamp(self.now_micros()));
                self.request_redraw();
            }
            WindowEvent::CursorEntered { .. } => {
                self.entry_pending = true;
            }
            WindowEvent::CursorLeft { .. } => {
                // Unhandled until now, which is why `:hover` never cleared when
                // the pointer left the window: the last row it was over stayed
                // highlighted for the rest of the session. A leave needs no
                // fresh position — the runtime treats it as "nothing is under
                // the pointer" whatever the hit test says — so the tracked
                // cursor is a record of where it went out, not a guess.
                let at = PhysicalPosition::new(self.cursor.0, self.cursor.1);
                let left = PointerConverter::new()
                    .left(PointerId::MOUSE, at, self.held_buttons)
                    .with_timestamp(self.now_micros());
                self.runtime.handle_pointer(left);
                self.request_redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Every modelled button, not just the left one. `Other(_)` is
                // the only one dropped, and dropped rather than folded onto a
                // modelled neighbour: a mouse's fourth thumb button is not a
                // middle click, and claiming it were would be a press the user
                // never made.
                if let Some(button) = PointerConverter::button(button) {
                    let event = self.pointer_button_event(button, state);
                    self.runtime.handle_pointer(event);
                    self.request_redraw();
                }
            }
            WindowEvent::Focused(_) => {
                // Deliberately nothing, and named here so it reads as a decision
                // rather than an oversight. Window activation is a concept the
                // runtime does not have: `UiRuntime` models NODE focus, and the
                // only door into it moves focus, fires `blur`/`focus` and
                // commits the outgoing field's `change`. Routing a deactivated
                // window through that door would empty the field's caret and
                // commit it because the user alt-tabbed — the browsers this
                // toolkit takes its vocabulary from keep `activeElement` across
                // exactly that.
                //
                // Doing it right needs a window-activation state on the runtime
                // and a selector that reads it (a `:window-inactive`-shaped
                // one), so an inactive window can dim its selection and rest its
                // caret without losing either. None of that exists, and
                // inventing it here would put the state in the adapter, which is
                // the one place it must not live.
                //
                // The one cost of ignoring this that would have bitten is
                // already paid elsewhere: a modifier released while the window
                // was unfocused is corrected by `ModifiersChanged` below.
            }
            WindowEvent::Touch(contact) => {
                // A finger enters through the same door as the mouse: the runtime's
                // hit-testing, hover, focus and activation are device-agnostic and
                // already read `PointerDevice`, so touch needs the conversion and
                // nothing else. The contact carries its own location, so — unlike
                // `MouseInput`, which carries none — it never consults or updates
                // the tracked cursor; a tap must not teleport the wheel's target.
                let event = PointerConverter::new()
                    .touch(contact)
                    .with_timestamp(self.now_micros());
                self.runtime.handle_pointer(event);
                self.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // A wheel notch scrolls the `overflow: scroll` container under the
                // cursor. winit reports whole lines or raw pixels; normalize both to
                // a pixel step, and invert so that wheel-down (negative lines)
                // increases scroll_top — i.e. reveals content further down. The
                // cursor is passed in the same (physical) space the pointer
                // converter uses, so the hit that finds the container matches the
                // one a click would make.
                #[allow(clippy::cast_possible_truncation)]
                let at = Point::new(self.cursor.0 as f32, self.cursor.1 as f32);
                // Converted whole — both axes, the platform's own unit, and the
                // trust this boundary confers — rather than flattened to one
                // float. The line-to-pixel policy lives on the runtime now, so
                // the fallback below asks for the same answer instead of
                // keeping a second copy of the constant.
                let wheel = PointerConverter::wheel(delta, at).with_timestamp(self.now_micros());
                let step = wheel.pixels_y(UiRuntime::WHEEL_LINE_STEP);
                if self.runtime.handle_wheel(wheel).is_some() {
                    self.runtime.request_frame(FrameReason::WindowChange);
                    self.request_redraw();
                } else if let Some(handler) = self.scroll.as_mut() {
                    // No `overflow: scroll` container took it, so the app may want
                    // it: an app that windows its own list owns its scroll position.
                    handler(step / self.scale_factor.max(0.05) as f32);
                    self.runtime.request_frame(FrameReason::UiCommit);
                    self.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                // The platform's own answer, which beats the runtime's folded
                // one: a modifier released while this window was unfocused
                // produces no key event here, so the fold would keep believing
                // it is held. Alt-tab away, let go of Alt, come back.
                self.runtime
                    .set_modifiers(KeyboardConverter::modifiers(state.state()));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let WinitPhysicalKey::Code(code) = event.physical_key {
                    if let Some(physical) = KeyboardConverter::physical_key(code) {
                        let logical = KeyboardConverter::logical_key(&event.logical_key);
                        let pressed = event.state == ElementState::Pressed;
                        // Trusted here and nowhere else, the way the pointer
                        // converter is the single `as_trusted` call site for a
                        // pointer: this is the platform boundary, the OS event is
                        // on the other side of it, and everything that reaches
                        // `handle_key` by any other road stays untrusted.
                        let mut key = KeyEvent::new(physical, logical, pressed)
                            .with_timestamp(self.now_micros())
                            .as_trusted();
                        if event.repeat {
                            key = key.as_repeat();
                        }
                        self.runtime.handle_key(key);
                        self.request_redraw();
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
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => self.render_frame(),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
        self.sync_ime();
    }

    #[doc = include_str!("runner.doc/about_to_wait.md")]
    #[doc = include_str!("runner.doc/user_event.md")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        // The acquire thread has an image. The frame this loop skipped for want
        // of one can now be drawn.
        self.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.runtime.poll_resource_tasks() > 0 {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
        // A request queued from outside the input path — a stylesheet replaced when a
        // timer fired, or a background task's result — has to reach the runtime even
        // though no event arrived, and `apply` runs inside a frame. So an idle loop
        // owes a turn exactly when something is queued. Deliberately not "when a
        // registration is still unresolved": a name that matches nothing never
        // resolves, and waking for that would spin forever over one typo.
        if self.control.has_pending() {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
    }
}

#[doc = include_str!("runner.doc/init_gpu.md")]
fn init_gpu(
    window: &Arc<Window>,
) -> Result<
    (
        wgpu::Surface<'static>,
        wgpu::SurfaceConfiguration,
        wgpu::Device,
        wgpu::Queue,
        wgpu::AdapterInfo,
    ),
    String,
> {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(window.clone())
        .map_err(|error| error.to_string())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        // Not `Limits::default()`: those are the desktop tier, and asking for
        // them on a device that cannot meet one of them fails the whole
        // request with nothing on screen and nothing to read. The renderer
        // needs what the adapter has, so ask for exactly that.
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())?;
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or_else(|| "surface unsupported by adapter".to_owned())?;
    // Let the root's own `background-color` decide whether the window is see-through,
    // with no API to opt into: `get_default_config` picks `Opaque`, which throws the
    // alpha away at composite time, so an `rgba(…, 0.7)` root would paint correctly
    // and still show up solid. Asking for a premultiplied surface hands the alpha to
    // the compositor instead — Vello already clears to `TRANSPARENT`, so an opaque
    // root still fills every pixel with alpha 1 and looks exactly as before. This is
    // therefore not a mode: it is the absence of one, and the only thing that decides
    // the result is the stylesheet. Where the compositor offers neither premultiplied
    // nor postmultiplied alpha the default stands and the window stays opaque.
    let alpha_modes = surface.get_capabilities(&adapter).alpha_modes;
    for mode in [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
    ] {
        if alpha_modes.contains(&mode) {
            config.alpha_mode = mode;
            break;
        }
    }
    // `get_default_config` picks `Fifo` — vsync — which is the right default: it
    // paces to the display and burns no power on frames nobody sees. But it also
    // makes `present` block until the next refresh, so a profiling run cannot tell
    // real GPU cost from waiting, and a frame that misses the deadline reads the
    // same as one that is merely paced. `OBSCUR_NO_VSYNC=1` unpaces the swapchain
    // so a measurement shows the true frame cost.
    if std::env::var_os("OBSCUR_NO_VSYNC").is_some() {
        let supported = surface.get_capabilities(&adapter).present_modes;
        for mode in [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox] {
            if supported.contains(&mode) {
                config.present_mode = mode;
                break;
            }
        }
        println!("[present-mode] {:?}", config.present_mode);
    }
    Ok((surface, config, device, queue, adapter.get_info()))
}

#[doc = include_str!("runner.doc/held_after.md")]
fn held_after(held: PointerButtons, button: PointerButton, state: ElementState) -> PointerButtons {
    let mut held = held;
    match state {
        ElementState::Pressed => held.press(button),
        ElementState::Released => held.release(button),
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A chord is a real state, and winit never reports one: it sends a
    /// transition per button and leaves the set to whoever is counting. Before
    /// this the runner counted nothing — a press was always exactly
    /// `{PRIMARY}` and a release always `{}` — so two buttons held at once was
    /// unrepresentable and, since the guard dropped every button but the left,
    /// unreachable as well.
    #[test]
    fn two_buttons_held_at_once_are_two_buttons_held_at_once() {
        let mut held = PointerButtons::new();

        held = held_after(held, PointerButton::Primary, ElementState::Pressed);
        assert!(held.contains(PointerButton::Primary));
        assert!(!held.contains(PointerButton::Secondary));

        held = held_after(held, PointerButton::Secondary, ElementState::Pressed);
        assert!(
            held.contains(PointerButton::Primary) && held.contains(PointerButton::Secondary),
            "the second press must join the first, not replace it"
        );

        // And releasing one leaves the other, which is the half a per-event
        // guess gets wrong: the set is not "the button that just moved".
        held = held_after(held, PointerButton::Primary, ElementState::Released);
        assert!(!held.contains(PointerButton::Primary));
        assert!(
            held.contains(PointerButton::Secondary),
            "the button still down is still down"
        );

        held = held_after(held, PointerButton::Secondary, ElementState::Released);
        assert!(held.is_empty());
    }

    /// The set an event carries is the one held *after* the transition — a
    /// press includes itself, a release excludes itself. That is what
    /// `PointerEvent::buttons` means, and it is why the press half of the old
    /// hardcoding (`{PRIMARY}`, then `{}`) looked right for one button.
    #[test]
    fn the_set_is_taken_after_the_transition_not_before() {
        let pressed = held_after(
            PointerButtons::new(),
            PointerButton::Middle,
            ElementState::Pressed,
        );
        assert!(pressed.contains(PointerButton::Middle));

        let released = held_after(pressed, PointerButton::Middle, ElementState::Released);
        assert!(released.is_empty());
    }

    /// A release of a button that was never pressed — the window was not
    /// focused when it went down, or the fold started mid-chord — must not
    /// corrupt the set. It clears a bit that is already clear.
    #[test]
    fn releasing_an_unheld_button_is_inert() {
        let mut held = PointerButtons::new();
        held.press(PointerButton::Primary);
        let after = held_after(held, PointerButton::Forward, ElementState::Released);
        assert_eq!(after, held);
    }
}

#[doc = include_str!("runner.doc/gpu_diagnostic.md")]
fn gpu_diagnostic(info: &wgpu::AdapterInfo) -> Diagnostic {
    // Reported on success, because "which backend did wgpu settle on" is the
    // first question behind a slow or blank surface and nothing else answers it:
    // a Vulkan initialisation that quietly fell back to GL looks, from here,
    // exactly like a Vulkan one that is simply slow.
    Diagnostic::new(
        DiagnosticCode::new("render.adapter"),
        DiagnosticSeverity::Info,
        DiagnosticSubsystem::Render,
        format!(
            "{:?} on {} ({:?}), driver {} {}",
            info.backend, info.name, info.device_type, info.driver, info.driver_info
        ),
    )
}

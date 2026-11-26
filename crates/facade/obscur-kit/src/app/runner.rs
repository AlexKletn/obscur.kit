#![doc = include_str!("../../docs/api/app/runner.doc/_module.md")]

mod frame_preparation;
#[cfg(target_os = "linux")]
mod opaque_scene;
#[cfg(target_os = "linux")]
mod touch_window_resize;
mod touchpad_transform;
mod transform_component;
#[cfg(target_os = "linux")]
mod wayland_pacing;
#[cfg(feature = "accesskit-window")]
mod window_accessibility;
mod window_actions;
mod window_capture;
mod window_commit;
mod window_event_handler;
mod window_pointer_input;
mod window_resize;
mod window_text_input;
use frame_preparation::PreparedFrame;
use window_commit::WindowCommit;

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::Window;

use super::FrameInfo;
use super::app_parts::AppParts;
use super::frame_stats::FrameStats;
#[cfg(target_os = "linux")]
use super::presentation_stats::{PresentationStats, unpaced_wayland_redraw};
use super::window_size_limits::WindowSizeLimits;
use super::window_wake::WindowWake;
use super::{AppError, RenderBackend};
use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::control::{AppControl, WindowState};
use crate::geometry::{Point, Size};
use crate::interaction::pointer::PointerButtons;
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render_vello::renderer::{SurfaceAcquirer, VelloRenderer};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::runtime::lifecycle::ShutdownConcern;
use crate::view::view::View;

#[doc = include_str!("../../docs/api/app/runner.doc/MAX_FRAME_DELTA.md")]
const MAX_FRAME_DELTA: f32 = 0.1;

/// Builds the runtime, forwards the app's root builder + command handlers + CSS,
/// then opens the window and runs the event loop to completion.
pub(super) fn run(
    parts: AppParts,
    builder: Box<dyn FnMut() -> View + 'static>,
) -> Result<(), AppError> {
    run_inner(parts, builder, None)
}

#[doc = include_str!("../../docs/api/app/runner.doc/run_with_frame.md")]
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
    mut parts: AppParts,
    builder: Box<dyn FnMut() -> View + 'static>,
    frame_info: Option<Rc<Cell<FrameInfo>>>,
) -> Result<(), AppError> {
    // Read before `boot` consumes the parts. A layer surface has no title and no
    // requested size — the platform decides both — so these belong to the window
    // runner rather than to the shared `BootedApp`, where they would sit unread in
    // a build with no windowing backend.
    let (title, app_id) = (parts.title.clone(), parts.app_id.clone());
    let size_limits = parts.size_limits;
    let window_mode = parts.window_mode;
    let size = size_limits.constrain_initial(parts.size)?;
    let viewport = Size::new(size.0, size.1);
    if let Some(info) = &frame_info {
        info.set(FrameInfo {
            viewport,
            scale: 1.0,
        });
    }
    let decorated = parts.decorated;
    let resize = parts.resize;
    let prefer_opengl = parts.prefer_opengl;
    let render_backend = RenderBackend::resolve(parts.render_backend)?;
    let background_blur = parts.background_blur;
    let window_waker = parts.window_waker.take();
    let mut booted = parts.boot(builder, viewport)?;

    // Built before the runner: the acquire thread needs something to wake this
    // loop with, and `Window::request_redraw` is not it — it marks the window
    // dirty without promising to rouse a loop parked in `wait`, which is exactly
    // where a frame skipped for want of a swapchain image leaves it.
    let event_loop = match EventLoop::<WindowWake>::with_user_event().build() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            booted
                .runtime
                .lifecycle_mut()
                .report_critical_stop(ShutdownConcern::EventLoop);
            booted
                .control
                .publish_lifecycle_events(booted.runtime.release().into_events());
            return Err(AppError::EventLoop(error));
        }
    };
    let proxy = event_loop.create_proxy();
    if let Some(ready) = window_waker {
        let frame_proxy = proxy.clone();
        ready(Box::new(move || {
            frame_proxy.send_event(WindowWake::ViewChanged).is_ok()
        }));
    }

    if let Some(appearance) = &mut booted.appearance {
        let proxy = proxy.clone();
        appearance.start(move || {
            let _ = proxy.send_event(WindowWake::AppearanceChanged);
        });
    }

    let mut app = AppRunner {
        touchpad_transform: touchpad_transform::TouchpadTransform::default(),
        render_backend,
        background_blur,
        #[cfg(target_os = "linux")]
        blur_request: None,
        #[cfg(target_os = "linux")]
        blur_commit_pending: None,
        #[cfg(target_os = "linux")]
        presentation: None,
        #[cfg(target_os = "linux")]
        wayland_gestures: None,
        #[cfg(target_os = "linux")]
        wayland_touch_move: None,
        #[cfg(target_os = "linux")]
        shown_sample: PresentationStats::default(),
        #[cfg(target_os = "linux")]
        wayland_pacer: None,
        detailed_stats: std::env::var_os("OBSCUR_FRAME_STATS").is_some_and(|mode| mode == "each"),
        frame_sequence: 0,
        last_commit: None,
        title,
        app_id,
        decorated,
        resize,
        prefer_opengl,
        resize_cursor: None,
        #[cfg(target_os = "linux")]
        touch_window_resize: None,
        size,
        size_limits,
        window_mode,
        runtime: booted.runtime,
        appearance: booted.appearance,
        css: booted.css,
        control: booted.control,
        #[cfg(feature = "accesskit-window")]
        accessibility: None,
        window: None,
        #[cfg(all(feature = "gamepad", target_os = "linux"))]
        gamepad_source: None,
        acquirer: None,
        extent: (0, 0),
        resize_acquire_pending: false,
        queued_resize: None,
        pending_commit: None,
        threaded_present: false,
        #[cfg(target_os = "linux")]
        resize_frame_pending: false,
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
        prepared_frame: None,
        prepare_ahead: std::env::var_os("OBSCUR_PREPARE_FRAME").is_some_and(|value| value == "1"),
        init_failure: None,
        scroll: booted.scroll,
        ime_allowed: false,
        frame_info,
        stats: std::env::var_os("OBSCUR_FRAME_STATS").map(|_| FrameStats::default()),
        fonts: None,
    };

    let result = event_loop.run_app(&mut app).map_err(AppError::EventLoop);
    for (_, complete) in app.control.take_frame_captures() {
        complete(crate::system::capture::CaptureResult::Failed(
            "window closed before the requested capture".into(),
        ));
    }
    let init_failure = app.init_failure.take();
    if let Some(error) = result.as_ref().err().or(init_failure.as_ref()) {
        let concern = match error {
            AppError::EventLoop(_) => ShutdownConcern::EventLoop,
            AppError::Window(_) | AppError::WindowCreation(_) => ShutdownConcern::Window,
            AppError::Gpu(_)
            | AppError::GpuSurface(_)
            | AppError::GpuAdapter(_)
            | AppError::GpuDevice(_)
            | AppError::GpuRenderer(_) => ShutdownConcern::Renderer,
            _ => ShutdownConcern::Runtime,
        };
        app.runtime.lifecycle_mut().report_critical_stop(concern);
    }
    if let Some(window) = &app.window {
        app.runtime
            .lifecycle_mut()
            .window_closed(crate::identity::WindowId::from_raw(window.id().into()));
    }
    let runtime = app.runtime.id();
    let mut events = app.runtime.lifecycle_mut().take_events();
    let control = app.control.clone();
    drop(app);
    events.push(crate::runtime::lifecycle::LifecycleEvent::ResourceReleaseComplete(runtime));
    control.update_window_state(None);
    control.publish_lifecycle_events(events);
    result?;

    // A GPU/window failure exits the loop cleanly on `resumed`, recording the
    // reason so `run` can report it rather than returning `Ok` on a blank window.
    match init_failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

/// Installs the real text stack over the host's system fonts (bundled last-resort
/// fallback appended by [`install_text_stack`](KitBootstrap::install_text_stack)),
/// and hands the renderer the *same* id→bytes assignment so scene font handles
/// line up with what it rasterizes — the codified fix for the id-matching problem.
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
        renderer.register_font(handle, std::sync::Arc::clone(bytes));
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

/// The winit application the runner drives: the window, the GPU renderer, the
/// swapchain surface, and the embeddable runtime, all created lazily on `resumed`.
struct AppRunner {
    touchpad_transform: touchpad_transform::TouchpadTransform,
    render_backend: RenderBackend,
    background_blur: bool,
    #[cfg(target_os = "linux")]
    blur_request: Option<obscur_wayland_effects::WindowBlur<Arc<Window>>>,
    #[cfg(target_os = "linux")]
    blur_commit_pending: Option<(bool, &'static str)>,
    #[cfg(target_os = "linux")]
    presentation: Option<obscur_wayland_effects::WindowPresentation<Arc<Window>>>,
    #[cfg(target_os = "linux")]
    wayland_gestures: Option<obscur_wayland_effects::WindowGestures<Arc<Window>>>,
    #[cfg(target_os = "linux")]
    wayland_touch_move: Option<obscur_wayland_effects::TouchWindowMove<Arc<Window>>>,
    #[cfg(target_os = "linux")]
    shown_sample: PresentationStats,
    #[cfg(target_os = "linux")]
    wayland_pacer: Option<wayland_pacing::WaylandPacer>,
    detailed_stats: bool,
    frame_sequence: u64,
    last_commit: Option<std::time::Instant>,
    title: String,
    app_id: String,
    decorated: bool,
    resize: bool,
    prefer_opengl: bool,
    resize_cursor: Option<winit::window::ResizeDirection>,
    #[cfg(target_os = "linux")]
    touch_window_resize: Option<touch_window_resize::TouchWindowResize>,
    size: (f32, f32),
    size_limits: WindowSizeLimits,
    window_mode: crate::WindowMode,
    runtime: UiRuntime,
    appearance: Option<super::appearance_subscription::AppearanceSubscription>,
    /// The CSS pipeline, alive for the app's lifetime because it owns the §4.12 parse
    /// cache: a replacement through [`AppControl::set_stylesheets`] re-parses only what
    /// changed identity, and a fresh pipeline per install would throw that away.
    css: CssPipeline,
    /// The application's handle onto this runtime. Everything it queued is applied
    /// once per committed frame — see [`AppRunner::render_frame`] for where, and
    /// [`AppControl`] for why it is a queue rather than an accessor.
    control: AppControl,
    #[cfg(feature = "accesskit-window")]
    accessibility: Option<window_accessibility::WindowAccessibility>,
    window: Option<Arc<Window>>,
    #[cfg(all(feature = "gamepad", target_os = "linux"))]
    gamepad_source: Option<obscur_window_winit::gamepad::GamepadSource>,
    #[doc = include_str!("../../docs/api/app/runner.doc/acquirer.md")]
    acquirer: Option<SurfaceAcquirer>,
    #[doc = include_str!("../../docs/api/app/runner.doc/extent.md")]
    extent: (u32, u32),
    resize_acquire_pending: bool,
    queued_resize: Option<(u32, u32)>,
    pending_commit: Option<WindowCommit>,
    threaded_present: bool,
    #[cfg(target_os = "linux")]
    resize_frame_pending: bool,
    #[doc = include_str!("../../docs/api/app/runner.doc/proxy.md")]
    proxy: Option<EventLoopProxy<WindowWake>>,
    renderer: Option<VelloRenderer>,
    /// The display's HiDPI device-pixel ratio (winit `scale_factor`).
    scale_factor: f64,
    /// Last cursor position in physical px, so a `MouseInput` (which carries none)
    /// dispatches at the cursor.
    cursor: (f64, f64),
    #[doc = include_str!("../../docs/api/app/runner.doc/held_buttons.md")]
    held_buttons: PointerButtons,
    #[doc = include_str!("../../docs/api/app/runner.doc/entry_pending.md")]
    entry_pending: bool,
    /// The instant the app started, and the origin of every pointer event's
    /// logical timestamp — see [`now_micros`](AppRunner::now_micros).
    epoch: std::time::Instant,
    /// When the previous frame ran, so the next one can advance motion by the
    /// time that actually elapsed. `None` until the first frame.
    last_frame: Option<std::time::Instant>,
    prepared_frame: Option<PreparedFrame>,
    prepare_ahead: bool,
    /// A window/GPU failure recorded on `resumed` (which cannot return a `Result`)
    /// so [`run`] can surface it instead of returning `Ok` on a blank window.
    init_failure: Option<AppError>,
    /// The app's leftover-wheel handler, if it registered one.
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    #[doc = include_str!("../../docs/api/app/runner.doc/ime_allowed.md")]
    ime_allowed: bool,
    /// The viewport snapshot a frame-aware builder reads, refreshed on every
    /// surface change.
    frame_info: Option<Rc<Cell<FrameInfo>>>,
    /// Per-phase frame timings, present only when `OBSCUR_FRAME_STATS=1` is set.
    stats: Option<FrameStats>,
    /// The renderer's font hand-off table, kept so faces the text stack pulls in
    /// from the host mid-run can be handed over too — see
    /// [`InstalledFonts::take_discovered`](crate::bootstrap::InstalledFonts::take_discovered).
    fonts: Option<crate::bootstrap::InstalledFonts>,
}

impl AppRunner {
    fn native_touchpad_transform(
        &mut self,
        component: transform_component::TransformComponent,
        phase: winit::event::TouchPhase,
    ) {
        let position = Point::new(self.cursor.0 as f32, self.cursor.1 as f32);
        let now = self.now_micros();
        if let Some(event) = self
            .touchpad_transform
            .update(component, phase, position, now)
            && self.runtime.handle_touchpad_gesture(event)
        {
            self.request_redraw();
        }
    }

    #[doc = include_str!("../../docs/api/app/runner.doc/now_micros.md")]
    #[allow(clippy::cast_possible_truncation)]
    fn now_micros(&self) -> u64 {
        self.epoch.elapsed().as_micros() as u64
    }

    /// The device-pixel scale the runtime renders at: the display density, floored
    /// so a degenerate `0` never divides the viewport.
    #[allow(clippy::cast_possible_truncation)]
    fn scale(&self) -> f32 {
        (self.scale_factor as f32).max(0.05)
    }

    /// Points the runtime's viewport + device scale at the swapchain's physical
    /// extent and the display density — the kit's HiDPI mapping in one call.
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

    /// Publish native window state and redraw views on changes to window mode or focus.
    fn sync_window_state(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let scale_factor = window.scale_factor();
        let size = window.inner_size().to_logical::<f32>(scale_factor);
        self.control
            .update_window_screen_origin(window.inner_position().ok().map(|position| {
                crate::geometry::space::PointIn::new(Point::new(
                    position.x as f32,
                    position.y as f32,
                ))
            }));
        let next = WindowState {
            size: Size::new(size.width, size.height),
            scale_factor,
            maximized: window.is_maximized(),
            fullscreen: window.fullscreen().is_some(),
            minimized: window.is_minimized(),
            focused: window.has_focus(),
            resizable: window.is_resizable(),
        };
        let previous = self.control.window_state();
        if self.control.update_window_state(Some(next))
            && previous.is_none_or(|previous| {
                WindowState {
                    size: next.size,
                    scale_factor,
                    ..previous
                } != next
            })
        {
            self.runtime.request_frame(FrameReason::UiCommit);
            self.request_redraw();
        }
    }

    fn accept_window_extent(&mut self, extent: (u32, u32)) {
        if self.threaded_present {
            self.queued_resize = (extent != self.extent).then_some(extent);
        } else {
            self.apply_resize(extent);
        }
    }

    /// Select a new surface size and schedule a frame for it.
    fn apply_resize(&mut self, extent: (u32, u32)) {
        if self.extent == extent {
            return;
        }
        self.extent = extent;
        self.resize_acquire_pending = true;
        #[cfg(target_os = "linux")]
        {
            self.resize_frame_pending = true;
        }
        if let Some(acquirer) = self.acquirer.as_mut() {
            acquirer.resize(extent.0, extent.1);
        }
        self.sync_surface();
        self.runtime.request_frame(FrameReason::WindowChange);
        self.request_redraw();
    }

    /// Runs a scheduler-gated reactive frame, reads the committed scene, and
    /// presents it on-screen via the Vello offscreen-render + blit path. A lost
    /// swapchain is reconfigured and the frame skipped (the next redraw recovers).
    #[allow(clippy::cast_precision_loss)]
    fn render_frame(&mut self) {
        if self.pending_commit.is_some() {
            return;
        }
        #[cfg(target_os = "linux")]
        if self.wayland_frame_blocked() {
            return;
        }
        // The worker acquires the swapchain image without blocking event dispatch.
        let (image, acquire) = if self.frame_due() {
            let Some((image, acquire)) = self
                .acquirer
                .as_mut()
                .and_then(SurfaceAcquirer::try_take_timed)
            else {
                if let Some(acquirer) = self.acquirer.as_mut() {
                    acquirer.request();
                    if self.detailed_stats {
                        println!(
                            "[window-acquire-wait] next_id={} at_us={}",
                            self.frame_sequence + 1,
                            self.now_micros()
                        );
                    }
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
                    acquirer.request();
                }
                return;
            }
            (image, acquire)
        } else {
            return;
        };

        let prepared = self.prepared_frame.take();
        let prepared_ahead = prepared.is_some() && !self.prepared_frame_stale();
        let Some(PreparedFrame {
            snapshot,
            built,
            snapshotted,
            work,
        }) = (if prepared_ahead {
            prepared
        } else {
            self.prepare_frame()
        })
        else {
            return;
        };
        let present_at = std::time::Instant::now();

        #[cfg(target_os = "linux")]
        self.stage_window_occlusion(&snapshot);

        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        if super::gpu_instance::apply_renderer_request(
            &self.control,
            &mut self.runtime,
            renderer,
            &mut self.render_backend,
        ) {
            self.last_commit = None;
            self.stats = self.stats.take().map(|_| FrameStats::default());
            #[cfg(target_os = "linux")]
            self.shown_sample.restart(self.frame_sequence + 2);
        }
        let warmup = self.last_commit.is_none();

        // A character outside every registered face pulls one in from the host
        // while the frame is being built. The renderer rasterizes from its own
        // id→bytes table, so it has to be told before this frame is presented —
        // otherwise the glyph resolves, the run shapes, and nothing is drawn.
        // Empty on every frame where no new character appeared, which is nearly
        // all of them.
        if let Some(fonts) = self.fonts.as_ref() {
            for (handle, bytes) in fonts.take_discovered() {
                // The face itself, not a copy of it — see `layer_runner::init_gpu`'s
                // sibling loop and `VelloRenderer::register_font`.
                renderer.register_font(handle, bytes);
            }
        }

        // The extent came with the image: the surface is on the acquire thread,
        // and an image acquired before a resize must not be blitted against the
        // new numbers.
        let physical = Size::new(image.width as f32, image.height as f32);
        let submission = FrameSubmission::new(RenderFrame::new(snapshot, physical));
        window_capture::capture_frame(
            &self.control,
            self.window.as_deref(),
            &self.runtime,
            renderer,
            &submission,
            (image.width, image.height),
        );
        let window = self.window.as_ref();
        let frame = self.frame_sequence + 1;
        #[cfg(target_os = "linux")]
        let presentation = self.presentation.as_ref();
        #[cfg(target_os = "linux")]
        let pace_redraw = presentation.is_none()
            || self
                .wayland_pacer
                .as_ref()
                .map_or_else(|| !unpaced_wayland_redraw(), |pacer| pacer.is_suspended());
        #[cfg(not(target_os = "linux"))]
        let pace_redraw = true;
        let commit = WindowCommit {
            extent: self.extent,
            frame,
            present_at,
            acquire,
            built,
            snapshotted,
            work,
            warmup,
            prepared_ahead,
        };
        let acquirer = self.acquirer.as_mut();
        let mut queued = false;
        let outcome = renderer.present_to_texture_using(
            &submission,
            image,
            || {
                if pace_redraw && let Some(window) = window {
                    window.pre_present_notify();
                }
                #[cfg(target_os = "linux")]
                if let Some(presentation) = presentation {
                    presentation.request(frame);
                }
            },
            |texture| {
                if self.threaded_present
                    && let Some(acquirer) = acquirer
                {
                    match acquirer.queue_present(texture) {
                        Ok(()) => queued = true,
                        Err(texture) => texture.present(),
                    }
                } else {
                    texture.present();
                }
                !queued
            },
        );
        if queued {
            if self.detailed_stats {
                println!(
                    "[window-present-queued] id={frame} at_us={}",
                    self.now_micros()
                );
            }
            self.pending_commit = Some(commit);
        } else {
            if let Err(ref error) = outcome {
                eprintln!("present failed: {error:?}");
            }
            self.finish_commit(commit, std::time::Instant::now(), outcome.is_ok());
        }
    }

    /// Record a completed presentation and schedule any newer frame the runtime owes.
    fn finish_commit(
        &mut self,
        commit: WindowCommit,
        committed: std::time::Instant,
        succeeded: bool,
    ) {
        let WindowCommit {
            extent,
            frame,
            present_at,
            acquire,
            built,
            snapshotted,
            work,
            warmup,
            prepared_ahead,
        } = commit;
        #[cfg(not(target_os = "linux"))]
        let _ = extent;
        #[cfg(target_os = "linux")]
        {
            self.resize_frame_pending = self.extent != extent;
        }
        if succeeded {
            self.frame_sequence = frame;
            #[cfg(target_os = "linux")]
            if let Some(pacer) = self.wayland_pacer.as_mut() {
                pacer.committed(frame, present_at, committed, work);
            }
            if self.detailed_stats {
                let ms = |duration: std::time::Duration| duration.as_secs_f64() * 1000.0;
                let interval = self
                    .last_commit
                    .replace(committed)
                    .map_or(0.0, |previous| ms(committed - previous));
                let blur = if self.background_blur { "on" } else { "off" };
                let backend = self.render_backend.name();
                println!(
                    "[window-frame] id={frame} blur={blur} renderer={backend} warmup={warmup} prepared_ahead={prepared_ahead} commit_us={} acquire_ms={:.3} build_ms={:.3} \
                     snapshot_ms={:.3} present_ms={:.3} work_ms={:.3} interval_ms={interval:.3}",
                    committed.duration_since(self.epoch).as_micros(),
                    ms(acquire),
                    ms(built),
                    ms(snapshotted),
                    ms(committed - present_at),
                    ms(acquire + work + (committed - present_at)),
                );
            }
            #[cfg(target_os = "linux")]
            if let Some((enabled, protocol)) = self.blur_commit_pending.take() {
                self.runtime.report_diagnostic(&Diagnostic::new(
                    DiagnosticCode::new("window.background-blur.committed"),
                    DiagnosticSeverity::Info,
                    DiagnosticSubsystem::Render,
                    format!("{protocol}: {}", if enabled { "on" } else { "off" }),
                ));
            }
        }
        if let Some(stats) = self.stats.as_mut() {
            if self.last_commit.is_none() && succeeded {
                self.last_commit = Some(committed);
            }
            if !warmup && succeeded {
                stats.record(built, snapshotted, committed - present_at);
            }
        }
        // While motion (or a fling) remains, keep the loop turning. The runtime is
        // the authority on whether another frame is owed, so an idle UI still
        // sleeps — this is not a render-forever loop.
        if self.frame_due() {
            self.request_redraw();
        }
    }

    /// Stages the compositor blur change and records it for the next successful buffer commit.
    fn set_background_blur(&mut self, window: &Arc<Window>, enabled: bool) {
        #[cfg(target_os = "linux")]
        let previous = self.background_blur;
        self.background_blur = enabled;

        #[cfg(not(target_os = "linux"))]
        let _ = window;
        #[cfg(target_os = "linux")]
        {
            let protocol = if let Some(request) = self.blur_request.as_mut() {
                if enabled == previous {
                    return;
                }
                request.set_enabled(enabled);
                request.protocol()
            } else if enabled {
                match obscur_wayland_effects::WindowBlur::request(Arc::clone(window)) {
                    Ok(Some(request)) => {
                        let protocol = request.protocol();
                        self.blur_request = Some(request);
                        protocol
                    }
                    Ok(None) => {
                        self.runtime.report_diagnostic(&Diagnostic::new(
                            DiagnosticCode::new("window.background-blur.unavailable"),
                            DiagnosticSeverity::Info,
                            DiagnosticSubsystem::Render,
                            "No compositor blur protocol available for this window",
                        ));
                        return;
                    }
                    Err(error) => {
                        self.runtime.report_diagnostic(&Diagnostic::new(
                            DiagnosticCode::new("window.background-blur"),
                            DiagnosticSeverity::Warning,
                            DiagnosticSubsystem::Render,
                            error,
                        ));
                        return;
                    }
                }
            } else {
                return;
            };
            self.blur_commit_pending = Some((enabled, protocol));
            self.runtime.report_diagnostic(&Diagnostic::new(
                DiagnosticCode::new("window.background-blur.staged"),
                DiagnosticSeverity::Info,
                DiagnosticSubsystem::Render,
                format!("{protocol}: {}", if enabled { "on" } else { "off" }),
            ));
        }
    }

    #[doc = include_str!("../../docs/api/app/runner.doc/sync_ime.md")]
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
}

#[doc = include_str!("../../docs/api/app/runner.doc/init_gpu.md")]
fn init_gpu(
    window: &Arc<Window>,
    prefer_opengl: bool,
) -> Result<
    (
        wgpu::Surface<'static>,
        wgpu::SurfaceConfiguration,
        wgpu::Device,
        wgpu::Queue,
        wgpu::AdapterInfo,
    ),
    AppError,
> {
    // `layer_runner::init_gpu` carries the full note: what `InstanceFlags`
    // costs a frame and why it stays on, and what asking for every backend
    // costs in resident memory and DRM handles. Both decisions live in
    // `gpu_instance::open`, and with no environment variable set the flags are
    // bit-for-bit the ones this had.
    let open_default =
        || super::gpu_instance::open(|instance| instance.create_surface(window.clone()));
    let (_instance, surface, adapter) = if prefer_opengl {
        super::gpu_instance::open_with(
            [wgpu::Backends::GL],
            || wgpu::InstanceDescriptor::new_with_display_handle(Box::new(window.clone())),
            |instance| instance.create_surface(window.clone()),
        )
        .ok()
        .filter(|(_, _, adapter)| adapter.get_info().device_type != wgpu::DeviceType::Cpu)
        .map_or_else(open_default, Ok)
    } else {
        open_default()
    }?;
    let (device, queue) = super::gpu_instance::request_device(&adapter)?;
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .ok_or_else(|| AppError::Gpu("surface unsupported by adapter".to_owned()))?;
    if adapter.get_info().backend == wgpu::Backend::Gl {
        // OpenGL can use a linear view only when the surface itself uses
        // the matching unorm format.
        config.format = config.format.remove_srgb_suffix();
    }
    super::gpu_instance::apply_frame_latency(&mut config);
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

/// An informational diagnostic naming the adapter and backend wgpu chose.
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

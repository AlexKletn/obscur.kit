#![doc = include_str!("layer_runner.doc/_module.md")]

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use std::sync::Arc;

use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{
    AcquireWake, SurfaceAcquirer, VelloDevice, VelloRenderer, VelloSurface,
};
use crate::runtime::frame::FrameReason;
use crate::shell_layer::event::LayerInput;
use crate::shell_layer::surface::{LayerWaker, LayerWindow};
use crate::view::view::View;
use crate::windowing::layer::{LayerSettings, LayerSurfaceEvent, LayerSurfaceState};
use crate::windowing::port::LayerBackend;

use super::AppError;
use super::app_parts::AppParts;
use super::layer_control::LayerControl;
use super::layer_loop_stats::LayerLoopStats;
use super::turn_clock::TurnClock;

#[doc = include_str!("layer_runner.doc/MAX_FRAME_DELTA.md")]
const MAX_FRAME_DELTA: f32 = 0.1;

#[doc = include_str!("layer_runner.doc/ANIMATION_STEP_MS.md")]
const ANIMATION_STEP_MS: u64 = 16;

#[doc = include_str!("layer_runner.doc/run.md")]
pub(super) fn run(
    parts: AppParts,
    settings: &LayerSettings,
    builder: Box<dyn FnMut() -> View + 'static>,
    layer_control: &LayerControl,
    waker: Option<&LayerWaker>,
) -> Result<(), AppError> {
    let mut window = LayerWindow::open_with(settings, &parts.app_id, waker)?;

    // The protocol requires an initial commit with no buffer; the compositor
    // answers with the real size. Drawing before that is a protocol error, not a
    // race, so the first frame waits here.
    while !window.is_configured() && !window.is_closed() {
        window.dispatch(None)?;
    }
    // Whatever arrived during the wait was the configure itself; the loop below
    // starts from the geometry the backend now reports, not from a replay.
    let _ = window.poll_events();
    if window.is_closed() {
        layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Closed);
        return Ok(());
    }
    layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Ready);

    let booted = parts.boot(builder, window.size());
    let mut runtime = booted.runtime;
    let boot = booted.boot;
    let mut scroll = booted.scroll;
    // The application's live-control queue and the pipeline it installs sheets
    // through. Both are driven below on exactly the two beats the windowed runner
    // drives them on; a runner that took neither would leave every positional
    // binding unresolved and every post-boot stylesheet unapplied.
    let mut css = booted.css;
    let app_control = booted.control;

    let (mut renderer, surface, device, _queue, gpu) = init_gpu(&window)?;
    runtime.report_diagnostic(&gpu_diagnostic(&gpu));
    let installed = install_text(&boot, &mut runtime, &mut renderer);

    // The blocking acquire leaves this thread. `get_current_texture` waits for
    // the compositor to release a buffer, and while it waits nothing reads the
    // Wayland socket — touches were measured arriving 544 ms late, in bursts
    // (audit 1-D). The spec calls that out in three places: platform calls are
    // to be asynchronous, the synchronous frame phase must stay short, and
    // hidden synchronous blocking is grounds for redesign.
    //
    // Its own waker, not the application's: ringing that one means "reconsider
    // the interface" and would cost a frame. This one means only that the wait
    // in `dispatch` may end.
    let turn_waker = LayerWaker::new().ok();
    if let Some(turn) = turn_waker.as_ref() {
        window.add_turn_waker(turn)?;
    }
    let wake: Option<AcquireWake> = turn_waker.map(|turn| {
        let turn = turn.clone();
        Arc::new(move || turn.wake()) as AcquireWake
    });
    let mut acquirer = SurfaceAcquirer::spawn(surface, device.clone(), wake);

    // Declare how the buffer relates to logical pixels before the first frame,
    // or the compositor reads the physical buffer as a logical size and the
    // panel comes out at double extent with everything drawn twice as large.
    window.sync_scale();
    runtime.set_surface(window.physical_size(), window.scale());
    // And configure the swapchain *after* that declaration, not before it. The
    // surface was already configured once, inside `init_gpu`, while the buffer
    // still said nothing about its scale — and the audit's 1-E is a panel that
    // opens at double extent and stays there until the first reconfigure, which
    // is the only other thing that reaches `VelloSurface::resize`. The runner
    // this replaced got the same configure for free: it left the initial
    // configure event in the queue, so its first turn ran the resize path.
    resize_to_surface(&mut acquirer, &window);
    runtime.request_frame(FrameReason::UiCommit);

    // Time is read once per turn and the mark moves with the reading. The loop
    // advances motion on every turn, not only on the turns that end in a frame,
    // so a clock that moved only with the frames would hand the same interval
    // out twice — see [`TurnClock`].
    let mut clock = TurnClock::new(MAX_FRAME_DELTA);
    // Off unless asked. The three phases are timed separately because the whole
    // question this answers is which of them is the long one — and the runtime's
    // own `[phase-stats]` cannot answer it, since everything from rasterising
    // onward happens after `run_due_frame_reactive` returns (audit 3-D).
    let mut loop_stats = LayerLoopStats::enabled();
    // When the previous frame reached the screen, on the compositor's clock.
    let mut last_shown: u64 = 0;
    while !window.is_closed() && !layer_control.close_requested() {
        // Idle sleeps until the compositor — or a `LayerWaker` — has something
        // to say. Animating bounds the wait, so a frame happens whether or not
        // anything arrived. The loop turns for both without a thread to spare.
        // Asked for before the wait, so the image is being fetched while the
        // loop is doing everything else. By the time the frame is built it has
        // usually landed, and if it has not the loop skips rather than waits.
        if runtime.has_pending_frame() {
            acquirer.request();
        }

        let timeout = runtime
            .has_pending_frame()
            .then(|| std::time::Duration::from_millis(ANIMATION_STEP_MS));
        // An idle wait is not latency — the loop is asleep because there is
        // nothing to do — so only a bounded wait, taken while a frame is
        // pending, is worth counting against the input path.
        let waiting = loop_stats
            .as_ref()
            .and(timeout)
            .map(|_| std::time::Instant::now());
        window.dispatch(timeout)?;
        let waited = waiting.map_or(std::time::Duration::ZERO, |at| at.elapsed());

        let mut resized = false;
        for event in window.poll_events() {
            match event {
                LayerSurfaceEvent::Configured(_)
                | LayerSurfaceEvent::Resized(_)
                | LayerSurfaceEvent::ScaleFactorChanged(_) => {
                    window.sync_scale();
                    runtime.set_surface(window.physical_size(), window.scale());
                    resized = true;
                }
                LayerSurfaceEvent::Woken => {
                    // Whoever rang the waker wants the interface reconsidered;
                    // that is the only thing a wake can mean.
                    runtime.request_frame(FrameReason::UiCommit);
                }
                LayerSurfaceEvent::CloseRequested => break,
            }
        }
        for input in window.poll_input() {
            match input {
                LayerInput::Pointer(pointer) => {
                    runtime.handle_pointer(pointer);
                }
                LayerInput::Scroll { delta_y, at } => {
                    // Routed to the container under the pointer; a notch no
                    // container took reaches the application's own handler,
                    // which is what `App::on_scroll` promises on either backend.
                    let taken = runtime.handle_scroll(delta_y, at);
                    if taken.is_none()
                        && let Some(handler) = scroll.as_mut()
                    {
                        handler(delta_y);
                    }
                }
            }
        }
        if resized {
            resize_to_surface(&mut acquirer, &window);
            runtime.request_frame(FrameReason::WindowChange);
        }

        // Finished resource work commits HERE and nowhere else: both the loads
        // the frame phase deferred for want of an executor and the off-thread
        // ones whose results are waiting on the channel come back through this
        // one call, and it asks for the frame that shows them. Without it a
        // desktop surface requests an image or an SVG, the load runs, and the
        // decoded result is never applied — the element paints nothing for the
        // life of the process. The windowed runner has always drained it in
        // `about_to_wait`; this is the same drain on the turn this loop sleeps
        // at the end of.
        let _ = runtime.poll_resource_tasks();

        let delta = clock.elapsed(std::time::Instant::now());
        if delta > 0.0 {
            runtime.advance_motion(delta);
        }

        // A request queued since the last frame — a replaced stylesheet, a queued
        // runtime action — reaches the frame that is about to be built rather than
        // the one after it. Guarded because the steady state is "nothing queued".
        if app_control.has_pending() {
            app_control.apply(&mut runtime, &mut css);
        }

        let mut build = std::time::Duration::ZERO;
        let mut present = std::time::Duration::ZERO;
        // The image FIRST, and the frame only if one arrived. Building before
        // asking would throw the work away on every turn that finds nothing
        // ready — and the pending frame must stay pending, which it does
        // precisely because `run_due_frame_reactive` is not called here.
        //
        // Nothing spins: the acquire thread rings the turn waker when the image
        // lands, which ends the wait in `dispatch` above. §3.5's "no busy loop"
        // and "stale intermediate frames may be skipped" are the same rule read
        // from two sides.
        let acquired = runtime
            .has_pending_frame()
            .then(|| acquirer.try_take())
            .flatten();
        // An image acquired at an extent the window no longer has is a frame
        // drawn at the wrong size, and nothing downstream can tell: the blit
        // scales whatever it is given. The image carries the extent it was taken
        // at precisely so the disagreement is caught here instead of on the
        // screen (audit 1-E). Dropping it costs one frame; the resize makes the
        // next one right, and the frame stays pending because it was never run.
        let acquired = acquired.and_then(|image| {
            let (width, height) = physical_extent(&window);
            if image.matches(width, height) {
                return Some(image);
            }
            drop(image);
            acquirer.resize(width, height);
            None
        });
        if let Some(image) = acquired {
            let built_at = std::time::Instant::now();
            let ran = runtime.run_due_frame_reactive().is_some();
            build = built_at.elapsed();
            if ran {
                // The one point in the frame where application code may touch the
                // runtime (spec §2.6, §4.12): the frame has committed, so the tree
                // exists and a `test-id` can resolve to a real node, and the next
                // frame has not started. Unconditional, because a binding waiting
                // for a tree is not a *pending request* — nothing would ask.
                app_control.apply(&mut runtime, &mut css);
                for (handle, bytes) in installed.take_discovered() {
                    renderer.register_font(handle, bytes.to_vec());
                }
                let frame = RenderFrame::new(runtime.snapshot(), window.physical_size());
                let submission = FrameSubmission::new(frame);
                // Asked before the commit it is about: a feedback binds to the
                // surface's next commit, and the next commit is the one wgpu is
                // about to make inside `present_to_texture`. Asking after would
                // measure the frame after this one.
                if loop_stats.is_some() {
                    window.ask_when_shown();
                }
                let presented_at = std::time::Instant::now();
                let _ = renderer.present_to_texture(&submission, image);
                present = presented_at.elapsed();
            }
        }
        // Applied here rather than at the top of the turn, because the handlers
        // that ask for it run *during* the frame above — waiting for the next
        // input event would leave the request sitting on an idle surface.
        if let Some(pending) = layer_control.take_pending() {
            // Asked BEFORE `apply` overwrites the surface's record of what it is
            // living under, because the question is what changed.
            //
            // A move is not a repaint. A margin, a band, a reservation, a
            // keyboard policy all change where the surface sits and nothing it
            // draws — and a drag is a stream of margin changes, so answering each
            // with a frame re-renders identical content once per position (audit
            // 2-H). An extent change still gets its frame, and a real resize
            // arrives from the compositor as a configure regardless.
            let redraw = pending.needs_redraw_from(window.settings());
            let _ = window.apply(&pending);
            if redraw {
                runtime.request_frame(FrameReason::WindowChange);
            }
        }
        if let Some(stats) = loop_stats.as_mut() {
            stats.record(waited, build, present);
            // The only timestamp in this loop that is not this process's guess:
            // the compositor saying a frame reached the screen, on the clock it
            // stamps input with. Printed raw, one line per frame, because the
            // number worth having is the gap between it and a touch — and that
            // subtraction belongs to whoever is reading, not here.
            for shown in window.take_presented() {
                let since = shown.at_micros.saturating_sub(last_shown);
                last_shown = shown.at_micros;
                println!(
                    "[presented] seq {} | at {} us | refresh {:.2} ms | since last {:.2} ms",
                    shown.seq,
                    shown.at_micros,
                    f64::from(shown.refresh_nanos) / 1_000_000.0,
                    since as f64 / 1000.0,
                );
            }
        }
        layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Ready);
        window.flush()?;
    }
    layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Closed);
    Ok(())
}

#[doc = include_str!("layer_runner.doc/physical_extent.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn physical_extent(window: &LayerWindow) -> (u32, u32) {
    let extent = window.physical_size();
    ((extent.width as u32).max(1), (extent.height as u32).max(1))
}

#[doc = include_str!("layer_runner.doc/resize_to_surface.md")]
fn resize_to_surface(acquirer: &mut SurfaceAcquirer, window: &LayerWindow) {
    let (width, height) = physical_extent(window);
    acquirer.resize(width, height);
}

#[doc = include_str!("layer_runner.doc/install_text.md")]
#[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
fn install_text(
    boot: &crate::bootstrap::KitBootstrap,
    runtime: &mut crate::runtime::UiRuntime,
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
    _boot: &crate::bootstrap::KitBootstrap,
    _runtime: &mut crate::runtime::UiRuntime,
    _renderer: &mut VelloRenderer,
) -> crate::bootstrap::InstalledFonts {
    crate::bootstrap::InstalledFonts::default()
}

#[doc = include_str!("layer_runner.doc/init_gpu.md")]
fn init_gpu(
    window: &LayerWindow,
) -> Result<
    (
        VelloRenderer,
        VelloSurface,
        wgpu::Device,
        wgpu::Queue,
        wgpu::AdapterInfo,
    ),
    AppError,
> {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(window.target())
        .map_err(|error| AppError::Gpu(error.to_string()))?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .map_err(|error| AppError::Gpu(error.to_string()))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        // Not `Limits::default()`: those are the desktop tier, and asking for
        // them on a device that cannot meet one of them fails the whole
        // request with nothing on screen and nothing to read. The renderer
        // needs what the adapter has, so ask for exactly that.
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .map_err(|error| AppError::Gpu(error.to_string()))?;

    let extent = window.physical_size();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (width, height) = ((extent.width as u32).max(1), (extent.height as u32).max(1));
    let mut config = surface
        .get_default_config(&adapter, width, height)
        .ok_or_else(|| AppError::Gpu("surface unsupported by adapter".to_owned()))?;
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

    // `get_default_config` picks `Fifo` — vsync — and on a desktop surface that
    // is not merely a pacing choice: `present` then blocks in
    // `get_current_texture` until the compositor releases a buffer, and this
    // loop is one thread, so for as long as it blocks the Wayland socket is
    // unread and touches queue in the kernel (audit 1-D, measured at up to
    // 544 ms). `OBSCUR_NO_VSYNC=1` unpaces the swapchain, which is how that
    // diagnosis is confirmed or refuted on a given machine.
    //
    // The windowed runner has had this branch all along. Its absence here was
    // the escape hatch existing on the path that does not need it and missing
    // from the one whose whole purpose is to be touched.
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

    let vello_device = VelloDevice::new(device.clone(), queue.clone())
        .map_err(|error| AppError::Gpu(error.to_string()))?;
    let renderer = VelloRenderer::new(RendererId::from_raw(2)).with_device(vello_device);
    let vello_surface = VelloSurface::new(surface, config);
    vello_surface.configure(&device);
    Ok((renderer, vello_surface, device, queue, adapter.get_info()))
}

#[doc = include_str!("layer_runner.doc/gpu_diagnostic.md")]
fn gpu_diagnostic(info: &wgpu::AdapterInfo) -> Diagnostic {
    // Not a failure, and reported anyway. Which backend and which adapter wgpu
    // settled on is the first thing anyone needs when a surface is slow or blank,
    // and it is the one thing nothing else can tell them: a Vulkan
    // initialisation that quietly fell back to GL looks, from the application's
    // side, exactly like a Vulkan one that is simply slow.
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

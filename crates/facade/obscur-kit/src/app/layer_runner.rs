//! The loop that drives a desktop surface.

use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

use std::sync::Arc;

use super::wayland_media_input::media_input_from_seats;
use crate::identity::NodeId;
use crate::render::frame::{FrameSubmission, RenderFrame};
use crate::render::renderer::RendererId;
use crate::render_vello::renderer::{AcquireWake, SurfaceAcquirer, VelloRenderer, VelloSurface};
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::shell_layer::event::LayerInput;
use crate::shell_layer::surface::{LayerWaker, LayerWindow};
use crate::view::view::View;
use crate::windowing::layer::{LayerSettings, LayerSurfaceEvent, LayerSurfaceState};
use crate::windowing::port::LayerBackend;
use crate::windowing::request::WindowOperationResult;

use super::AppError;
use super::app_parts::AppParts;
use super::layer_control::LayerControl;
use super::layer_loop_stats::LayerLoopStats;
use super::layer_popup_ui::LayerPopupUi;
use super::popup_request::PopupRequest;
use super::turn_clock::TurnClock;
use crate::windowing::layer::LayerPopupEvent;

#[doc = include_str!("../../docs/api/app/layer_runner.doc/MAX_FRAME_DELTA.md")]
const MAX_FRAME_DELTA: f32 = 0.1;

#[doc = include_str!("../../docs/api/app/layer_runner.doc/ANIMATION_STEP_MS.md")]
const ANIMATION_STEP_MS: u64 = 16;

#[doc = include_str!("../../docs/api/app/layer_runner.doc/STALLED_FRAME_MS.md")]
const STALLED_FRAME_MS: u64 = 100;

/// How long a surface may go without its first frame callback before the loop
/// says so on stderr.
///
/// Not a pacing figure and not the same thing as `ANIMATION_STEP_MS`: the
/// deadline decides when to build anyway, this decides when to *complain*, and
/// the two want opposite tolerances. Building again after 16 ms is right —
/// waiting longer would stall a surface a compositor never calls back.
/// Complaining after 16 ms is wrong: a compositor handed a new layer surface
/// commonly takes longer than one frame to composite it the first time, and the
/// line then fired on a healthy start and resolved itself, which it did on two
/// starts out of seven. Two seconds is far past any first composite and far
/// short of a human's patience, so a line that appears is a line worth reading.
const NO_CALLBACK_GRACE: std::time::Duration = std::time::Duration::from_secs(2);

/// Updates the runtime when the Wayland seat capabilities change.
fn update_media_input(window: &mut LayerWindow, runtime: &mut UiRuntime) {
    if let Some(capabilities) = window.take_pointing_capabilities() {
        runtime.set_detected_media_input(Some(media_input_from_seats(capabilities)));
    }
}

#[doc = include_str!("../../docs/api/app/layer_runner.doc/run.md")]
pub(super) fn run(
    parts: AppParts,
    settings: &LayerSettings,
    builder: Box<dyn FnMut() -> View + 'static>,
    layer_control: &LayerControl,
    waker: Option<&LayerWaker>,
) -> Result<(), AppError> {
    let mut render_backend = super::RenderBackend::resolve(parts.render_backend)?;
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

    let background_blur = parts.background_blur;
    let booted = parts.boot(builder, window.size())?;
    let mut runtime = booted.runtime;
    update_media_input(&mut window, &mut runtime);
    let mut appearance = booted.appearance;
    let mut panel_focus = PanelKeyboardFocus::default();
    let background_blur = booted
        .control
        .take_background_blur()
        .unwrap_or(background_blur);
    if background_blur && let Err(error) = window.request_background_blur() {
        runtime.report_diagnostic(&Diagnostic::new(
            DiagnosticCode::new("window.background-blur"),
            DiagnosticSeverity::Warning,
            DiagnosticSubsystem::Render,
            error.to_string(),
        ));
    }
    let boot = booted.boot;
    let mut scroll = booted.scroll;
    // The application's live-control queue and the pipeline it installs sheets
    // through. Both are driven below on exactly the two beats the windowed runner
    // drives them on; a runner that took neither would leave every positional
    // binding unresolved and every post-boot stylesheet unapplied.
    let mut css = booted.css;
    let app_control = booted.control;

    let (mut renderer, surface, device, _queue, gpu) = init_gpu(&window, render_backend)?;
    let max_dimension_px = device.limits().max_texture_dimension_2d;
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
        if let Some(appearance) = &mut appearance {
            let turn = turn.clone();
            appearance.start(move || turn.wake());
        }
    }
    let wake: Option<AcquireWake> = turn_waker.map(|turn| {
        let turn = turn.clone();
        Arc::new(move || turn.wake()) as AcquireWake
    });
    let mut acquirer = SurfaceAcquirer::spawn(surface, device.clone(), wake);
    if let Some(error) = acquirer.take_startup_error() {
        return Err(AppError::SurfaceAcquireThread(error));
    }

    // Declare how the buffer relates to logical pixels before the first frame,
    // or the compositor reads the physical buffer as a logical size and the
    // panel comes out at double extent with everything drawn twice as large.
    // And configure the swapchain *after* that declaration, not before it. The
    // surface was already configured once, inside `init_gpu`, while the buffer
    // still said nothing about its scale — and the audit's 1-E is a panel that
    // opens at double extent and stays there until the first reconfigure, which
    // is the only other thing that reaches `VelloSurface::resize`. The runner
    // this replaced got the same configure for free: it left the initial
    // configure event in the queue, so its first turn ran the resize path.
    resize_to_surface(&mut acquirer, &mut runtime, &window, max_dimension_px)?;
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
    // And where the screen was in its own count of refreshes when it did. See
    // `[presented]` below: the difference between the two numbers is the whole
    // of "did the panel not turn over, or did we not hand it anything".
    let mut last_msc: u64 = 0;
    // The frame gate. `Some(deadline)` means a frame has been committed and the
    // compositor has not yet said it is ready for another, so building one now
    // would be building it for nobody — and the deadline is when to stop
    // believing it will ever say so. See `STALLED_FRAME_MS`.
    let mut awaiting_callback: Option<std::time::Instant> = None;
    let mut callbacks_seen = false;
    let mut stall_reported = false;
    // When this surface first asked for a frame callback, which is the instant
    // the compositor started owing it one. `NO_CALLBACK_GRACE` is measured from
    // here and not from the first missed deadline: see the constant.
    let mut first_asked_callback: Option<std::time::Instant> = None;
    let mut parent_mapped = false;
    let mut popup_ui: Option<LayerPopupUi> = None;
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

        // How long this turn is allowed to sleep. Nothing pending: sleep until
        // something arrives. A frame pending and the compositor owing us a
        // callback: sleep until it comes or its deadline passes, whichever is
        // first. A frame pending and nothing owed: the old animation step, which
        // is now what it always claimed to be — a bound on an animating idle,
        // not a substitute for a display clock.
        let timeout = if layer_control.has_pending()
            || (parent_mapped && layer_control.has_pending_popup())
        {
            // A reconfiguration is applied at the TOP of the turn — see below —
            // so one queued after this turn's frame has to be able to reach the
            // next turn. Without this the loop would sleep on `None` with the
            // request still sitting in the queue, and a surface that asked to
            // move would simply not move until something else happened to wake
            // it. The zero timeout costs one immediate turn.
            Some(std::time::Duration::ZERO)
        } else if popup_ui.as_ref().is_some_and(LayerPopupUi::pending) {
            Some(std::time::Duration::from_millis(ANIMATION_STEP_MS))
        } else if runtime.has_pending_frame() {
            Some(awaiting_callback.map_or(
                std::time::Duration::from_millis(ANIMATION_STEP_MS),
                |deadline| deadline.saturating_duration_since(std::time::Instant::now()),
            ))
        } else {
            None
        };
        // An idle wait is not latency — the loop is asleep because there is
        // nothing to do — so only a bounded wait, taken while a frame is
        // pending, is worth counting against the input path.
        let waiting = loop_stats
            .as_ref()
            .and(timeout)
            .map(|_| std::time::Instant::now());
        window.dispatch(timeout)?;
        update_media_input(&mut window, &mut runtime);
        let waited = waiting.map_or(std::time::Duration::ZERO, |at| at.elapsed());

        let mut resized = false;
        for event in window.poll_events() {
            match event {
                LayerSurfaceEvent::Configured(_)
                | LayerSurfaceEvent::Resized(_)
                | LayerSurfaceEvent::ScaleFactorChanged(_) => {
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
        if resized {
            resize_to_surface(&mut acquirer, &mut runtime, &window, max_dimension_px)?;
        }
        for (input, activation) in window.poll_input_with_activation() {
            layer_control.set_popup_activation(if parent_mapped { activation } else { None });
            match input {
                LayerInput::TextInput(node, update) => {
                    runtime.apply_text_input_update(node, update)
                }
                LayerInput::Key(event) => {
                    runtime.handle_key(event);
                }
                LayerInput::Modifiers(modifiers) => runtime.set_modifiers(modifiers),
                LayerInput::KeyboardFocus(focused) => panel_focus.changed(&mut runtime, focused),
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
            layer_control.set_popup_activation(None);
        }
        if dispatch_popup_events(&mut window, layer_control, &mut popup_ui) {
            panel_focus.popup_closed(&mut runtime);
        }
        if parent_mapped && let Some(request) = layer_control.take_popup() {
            let prepare = |node| {
                runtime
                    .layout_boxes()
                    .get(&node)
                    .map(|layout| layout.bounds)
                    .ok_or_else(|| Box::<str>::from("the popup anchor has no laid-out box"))
            };
            let result = match request {
                PopupRequest::Show(node, side, frame) => prepare(node).and_then(|anchor| {
                    window
                        .show_popup(anchor, side, frame)
                        .map(|()| (None, None))
                        .map_err(|error| error.to_string().into())
                }),
                PopupRequest::View(node, side, size, content, activation) => prepare(node)
                    .and_then(|anchor| {
                        let (ui, first) =
                            LayerPopupUi::open(&boot, &css, size, content, runtime.media_input())?;
                        let (opened, menu_anchor) = if let Some(activation) = activation {
                            (
                                window.show_grabbed_popup(anchor, side, first, activation),
                                Some(node),
                            )
                        } else {
                            (window.show_popup(anchor, side, first), None)
                        };
                        opened
                            .map(|()| (Some(ui), menu_anchor))
                            .map_err(|error| error.to_string().into())
                    }),
                PopupRequest::Close => {
                    window.close_popup();
                    Ok((None, None))
                }
            };
            match result {
                Ok((next, menu_anchor)) => {
                    if dispatch_popup_events(&mut window, layer_control, &mut popup_ui) {
                        panel_focus.popup_closed(&mut runtime);
                    }
                    popup_ui = next;
                    panel_focus.menu_anchor = menu_anchor;
                }
                Err(error) => layer_control.dispatch_popup_event(LayerPopupEvent::Failed(error)),
            }
        }
        if parent_mapped
            && layer_control.take_popup_frame_request()
            && let Some(ui) = &mut popup_ui
        {
            ui.request_frame();
        }
        if resized {
            runtime.request_frame(FrameReason::WindowChange);
        }

        // The frame gate, opened here rather than before the wait because the
        // callback that opens it usually arrives *during* the wait.
        //
        // Either the compositor has said it is about to composite this surface
        // again, or the deadline for saying so has passed. The deadline is not a
        // pacing device; it is the promise that a surface a compositor never
        // calls back — hidden, animating from a snapshot, on another workspace —
        // comes back to life the moment it is on screen again, instead of
        // waiting forever for a signal that is not coming.
        if window.is_drawable_now() {
            callbacks_seen = true;
            awaiting_callback = None;
        } else if awaiting_callback.is_some_and(|at| std::time::Instant::now() >= at) {
            awaiting_callback = None;
        }
        // The single most useful line this loop can print, and it can only be
        // printed once the loop is asking. No frame callback has *ever* arrived
        // for this surface, which means the compositor is not compositing it —
        // so every frame built so far was built for nobody, and no amount of
        // making the frame cheaper will change that. Unconditional, not behind
        // the stats flag: this is a fault, not a measurement.
        //
        // Two things about *when*, and both were wrong before.
        //
        // It used to be printed on the first missed deadline, which is
        // `ANIMATION_STEP_MS` — 16 ms — while `callbacks_seen` is false. A
        // compositor handed a brand-new layer surface routinely takes longer
        // than one frame to composite it the first time, so the line appeared
        // on a healthy start (two starts in seven) and the next frame resolved
        // it. A false alarm about the one condition nothing else can tell you
        // about is worse than no line at all: the next person to meet the real
        // thing has already learnt to scroll past it. `NO_CALLBACK_GRACE` is
        // the whole fix — a surface that is genuinely not being composited is
        // still not being composited two seconds later.
        //
        // And it is asked here, on every turn, rather than inside the expiry
        // branch above. Expiries only recur while the loop keeps committing
        // frames; a surface whose host has nothing pending stops setting new
        // deadlines and would then never reach the test at all. The wait at the
        // top of the loop is bounded by `ANIMATION_STEP_MS`, so this is asked
        // at least sixty times a second whatever else is happening, and costs
        // one `Instant::elapsed` on the turns before the debt is settled.
        if !callbacks_seen
            && !stall_reported
            && first_asked_callback.is_some_and(|at| at.elapsed() >= NO_CALLBACK_GRACE)
        {
            stall_reported = true;
            eprintln!(
                "obscur-kit: no wl_surface.frame callback has arrived for this \
                 surface in {} s of asking; the compositor is not compositing \
                 it, and frames built now will not be shown",
                NO_CALLBACK_GRACE.as_secs(),
            );
        }
        let may_build = awaiting_callback.is_none();

        // Applied BEFORE the frame, not after it.
        //
        // Every one of these ends in a bare `wl_surface.commit`, and a bare
        // commit is still a content update: landing it on a surface whose last
        // frame the compositor has not shown yet replaces that frame with one
        // carrying no new buffer, and the presentation feedback bound to the
        // frame comes back `discarded`. Applied after the frame, as this was, a
        // host that reconfigures on input — a keyboard whose suggestion strip
        // appears and disappears as you type — overwrites its own frame once per
        // keystroke and pays in full for a frame nobody sees.
        //
        // The cost of moving it is one turn of latency on a geometry change,
        // which is bounded by the wait above. The cost of leaving it was every
        // frame during a reconfiguring gesture.
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
            let failure = match window.apply(&pending) {
                WindowOperationResult::Performed(_) => {
                    if redraw {
                        runtime.request_frame(FrameReason::WindowChange);
                    }
                    None
                }
                WindowOperationResult::Unsupported => {
                    Some("layer reconfiguration is unavailable".into())
                }
                WindowOperationResult::Failed(reason) => Some(reason),
            };
            if let Some(reason) = failure {
                runtime.report_diagnostic(&Diagnostic::new(
                    DiagnosticCode::new("window.layer.reconfigure"),
                    DiagnosticSeverity::Warning,
                    DiagnosticSubsystem::Platform,
                    reason,
                ));
            }
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
        runtime.poll_accessibility_actions();
        let _ = runtime.poll_resource_tasks();

        let delta = clock.elapsed(std::time::Instant::now());
        if delta > 0.0 {
            runtime.advance_motion(delta);
        }

        // A request queued since the last frame — a replaced stylesheet, a queued
        // runtime action — reaches the frame that is about to be built rather than
        // the one after it. Guarded because the steady state is "nothing queued".
        if let Some(appearance) = &mut appearance {
            appearance.refresh(&mut runtime);
        }
        if app_control.has_pending() {
            app_control.apply(&mut runtime, &mut css);
        }
        if let Some(ui) = &mut popup_ui {
            ui.set_detected_media_input(runtime.media_input());
            let update = ui.tick(&css, delta).and_then(|frame| {
                if let Some(frame) = frame {
                    window
                        .update_popup(frame)
                        .map_err(|error| error.to_string().into())
                } else {
                    Ok(())
                }
            });
            if let Err(error) = update {
                window.close_popup();
                if dispatch_popup_events(&mut window, layer_control, &mut popup_ui) {
                    panel_focus.popup_closed(&mut runtime);
                }
                layer_control.dispatch_popup_event(LayerPopupEvent::Failed(error));
            }
        }
        if let Some(enabled) = app_control.take_background_blur()
            && let Err(error) = window.set_background_blur(enabled)
        {
            runtime.report_diagnostic(&Diagnostic::new(
                DiagnosticCode::new("window.background-blur"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Render,
                error.to_string(),
            ));
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
        let acquired = (may_build && runtime.has_pending_frame())
            .then(|| acquirer.try_take())
            .flatten();
        // An image acquired at an extent the window no longer has is a frame
        // drawn at the wrong size, and nothing downstream can tell: the blit
        // scales whatever it is given. The image carries the extent it was taken
        // at precisely so the disagreement is caught here instead of on the
        // screen (audit 1-E). Dropping it costs one frame; the resize makes the
        // next one right, and the frame stays pending because it was never run.
        let acquired = if let Some(image) = acquired {
            let (width, height) = physical_extent(window.physical_size(), max_dimension_px)?;
            if image.matches(width, height) {
                Some(image)
            } else {
                drop(image);
                acquirer.resize(width, height);
                None
            }
        } else {
            None
        };
        if let Some(image) = acquired {
            super::gpu_instance::apply_renderer_request(
                &app_control,
                &mut runtime,
                &mut renderer,
                &mut render_backend,
            );
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
                    // The face itself, not a copy of it: the registry holds these
                    // bytes for the life of the process either way, and a CJK
                    // face is tens of megabytes.
                    renderer.register_font(handle, bytes);
                }
                let frame = RenderFrame::new(runtime.snapshot(), window.physical_size());
                let submission = FrameSubmission::new(frame);
                let presented_at = std::time::Instant::now();
                let timed = loop_stats.is_some();
                // Both requests are made from inside the present, in the one
                // instant between the last thing that can fail and the commit
                // itself — because both bind to "the surface's next commit", and
                // outside that instant nobody knows which commit that is.
                //
                // Asked here as it used to be asked, a render that failed left no
                // commit at all, and the feedback bound instead to whatever the
                // process committed next: a scale sync, a settings apply, the
                // frame after. Every subsequent answer would then be about a
                // different frame than the runner believed, permanently, with
                // nothing in the log to say so. The correct place is not "before
                // the call" but "before the commit", and only the renderer knows
                // where that is.
                let committed = std::cell::Cell::new(false);
                let outcome = renderer.present_to_texture_with(&submission, image, || {
                    window.ask_when_drawable();
                    if timed {
                        window.ask_when_shown();
                    }
                    committed.set(true);
                });
                present = presented_at.elapsed();
                if let Err(error) = outcome {
                    // Swallowed by a `let _ =` before this. A frame that was built
                    // and not presented is the most expensive thing this loop can
                    // do, and it was the one thing it did without a word.
                    eprintln!("obscur-kit: the frame was built and not presented: {error}");
                }
                if committed.get() {
                    parent_mapped = true;
                    // The compositor now owes this surface a frame callback, and
                    // until it arrives another frame would only overwrite this
                    // one. The deadline is generous once callbacks are known to
                    // arrive here and tight before that, so a compositor that
                    // never sends them costs one animation step, not a stall.
                    let fallback = if callbacks_seen {
                        STALLED_FRAME_MS
                    } else {
                        ANIMATION_STEP_MS
                    };
                    let now = std::time::Instant::now();
                    // The instant the compositor started owing this surface a
                    // callback, kept because `NO_CALLBACK_GRACE` is measured
                    // from it. Set on the first commit that asked and never
                    // again — every later commit renews the deadline, not the
                    // debt.
                    first_asked_callback.get_or_insert(now);
                    awaiting_callback = Some(now + std::time::Duration::from_millis(fallback));
                }
            }
        }
        if let Some(stats) = loop_stats.as_mut() {
            stats.record(waited, build, present, window.frame_callbacks());
            // Hand the present back to the runtime's phase clock so the NEXT
            // frame's `[frame]`/`[phase-stats]` line carries it. Until this
            // existed, the phases described the build and stopped there, and a
            // quarter of the frame — rasterise, acquire, blit, submit, commit —
            // was not in any subtotal at all (§3.0 M4); a reader looking for it
            // went hunting through `style` (audit 3-D). Written only under the
            // stats flag, on the thread that builds, so the un-instrumented loop
            // never touches it.
            // Only on a turn that actually presented. Most turns build nothing —
            // they wake, find no image or no pending frame, and leave `present`
            // at zero — and writing those zeros through would overwrite the last
            // real number before the next frame could read it, which is exactly
            // what the first cut of this did: every `[frame]` line said
            // `present 0.000` while `[turn]` showed 2.7 ms.
            //
            // The clock now TAKES what it reads rather than peeking, so this
            // guard no longer has to protect a value from being cleared by a
            // barren turn — it protects it from being cleared *early*. Between
            // this write and the next frame's read there may be many turns that
            // present nothing, and each of them must leave the number alone.
            if !present.is_zero() {
                crate::runtime::record_present_time(present);
            }
            // The only timestamp in this loop that is not this process's guess:
            // the compositor saying a frame reached the screen, on the clock it
            // stamps input with. Printed raw, one line per frame, because the
            // number worth having is the gap between it and a touch — and that
            // subtraction belongs to whoever is reading, not here.
            for shown in window.take_presented() {
                let since = shown.at_micros.saturating_sub(last_shown);
                last_shown = shown.at_micros;
                // `msc` is the screen's own count of refreshes, and the step
                // between two of them is the reading `seq` could never give:
                // `seq` counts what this process asked for, so it always steps by
                // one and always blames the panel. A step of one here means the
                // screen turned over as often as we handed it something, and
                // whatever is slow is below us; a step of N means the screen
                // offered N chances and we took one, and that is our side.
                //
                // Appended, never substituted: the existing four fields are what
                // `perf/report.py` reads, and a field it does not know about it
                // simply does not read.
                let step = shown.msc.saturating_sub(last_msc);
                last_msc = shown.msc;
                println!(
                    "[presented] seq {} | at {} us | refresh {:.2} ms | since last {:.2} ms \
                     | msc {} (+{}) | flags {:#04x}",
                    shown.seq,
                    shown.at_micros,
                    f64::from(shown.refresh_nanos) / 1_000_000.0,
                    since as f64 / 1000.0,
                    shown.msc,
                    step,
                    shown.flags,
                );
            }
            // On the same stream and in the same order as the presentations,
            // rather than on stderr where the two interleave by luck. The commit
            // count is the whole point of the line: `discarded` has one meaning
            // in the protocol and two causes, and this is what tells them apart.
            for dropped in window.take_discarded() {
                let cause = if dropped.commits_since_ask == 0 {
                    "no commit of ours came after it, so the compositor did not \
                     show it"
                } else {
                    "we committed over it before it was shown"
                };
                println!(
                    "[discarded] frame {} was never shown | commits since ask {} | {}",
                    dropped.seq, dropped.commits_since_ask, cause,
                );
            }
        }
        window.set_text_input(runtime.text_input_target().and_then(|node| {
            let selection = runtime.editing_selection()?;
            let rect = runtime
                .layout_boxes()
                .get(&node)
                .map(|layout| layout.bounds);
            Some((
                node,
                runtime.editing_text()?,
                selection.focus(),
                selection.anchor(),
                rect,
            ))
        }));
        layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Ready);
        window.flush()?;
    }
    window.close_popup();
    for event in window.poll_popup_events() {
        layer_control.dispatch_popup_event(event);
    }
    layer_control.publish(window.size(), window.scale(), LayerSurfaceState::Closed);
    Ok(())
}

#[doc = include_str!("../../docs/api/app/layer_runner.doc/physical_extent.md")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn physical_extent(
    extent: crate::geometry::Size,
    max_dimension_px: u32,
) -> Result<(u32, u32), AppError> {
    let checked_axis = |axis_px: f32| {
        (0.0..=f64::from(max_dimension_px))
            .contains(&f64::from(axis_px))
            .then(|| (axis_px as u32).max(1))
    };
    checked_axis(extent.width)
        .zip(checked_axis(extent.height))
        .ok_or_else(|| AppError::Gpu(format!(
            "layer buffer size {} × {} px is outside the GPU limit of {max_dimension_px} px per axis",
            extent.width, extent.height
        )))
}

#[doc = include_str!("../../docs/api/app/layer_runner.doc/resize_to_surface.md")]
fn resize_to_surface(
    acquirer: &mut SurfaceAcquirer,
    runtime: &mut UiRuntime,
    window: &LayerWindow,
    max_dimension_px: u32,
) -> Result<(), AppError> {
    let physical_size = window.physical_size();
    let (width, height) = physical_extent(physical_size, max_dimension_px)?;
    window.sync_scale()?;
    runtime.set_surface(physical_size, window.scale());
    acquirer.resize(width, height);
    Ok(())
}

#[cfg(test)]
mod physical_extent_tests {
    use super::physical_extent;
    use crate::geometry::Size;

    #[test]
    fn buffer_extents_respect_device_limits() {
        assert_eq!(
            physical_extent(Size::new(0.0, 8192.0), 8192).unwrap(),
            (1, 8192)
        );
        assert!(physical_extent(Size::new(8193.0, 32.0), 8192).is_err());
        assert!(physical_extent(Size::new(f32::INFINITY, 32.0), 8192).is_err());
    }
}

#[derive(Default)]
struct PanelKeyboardFocus {
    active: bool,
    previous: Option<NodeId>,
    menu_anchor: Option<NodeId>,
    returning: Option<NodeId>,
}

impl PanelKeyboardFocus {
    fn changed(&mut self, runtime: &mut UiRuntime, focused: bool) {
        self.active = focused;
        if focused {
            let previous = self.previous.take();
            let target = self.returning.take().or_else(|| {
                (runtime.focused().is_none() && self.menu_anchor.is_none())
                    .then_some(previous)
                    .flatten()
            });
            if target.is_some() && runtime.focus_node(target) {
                runtime.request_frame(FrameReason::Input);
            }
        } else {
            self.previous = runtime.focused();
            runtime.blur_window_focus();
        }
    }

    fn popup_closed(&mut self, runtime: &mut UiRuntime) {
        if let Some(anchor) = self.menu_anchor.take() {
            if self.active {
                if runtime.focus_node(Some(anchor)) {
                    runtime.request_frame(FrameReason::Input);
                }
            } else {
                self.returning = Some(anchor);
            }
        }
    }
}

#[cfg(test)]
mod popup_focus_tests {
    use super::*;
    use crate::geometry::Size;
    use crate::identity::{ComponentId, RuntimeId};
    use crate::view::view::ViewNode;

    #[test]
    fn dismissing_a_menu_waits_for_native_focus_before_restoring_its_anchor() {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(80.0, 80.0));
        runtime.render_now(
            &View::single(ViewNode::element(ComponentId::from_raw(1))),
            &[],
        );
        let root = runtime.tree().root().unwrap();
        let anchor = runtime.tree().get(root).unwrap().child_nodes()[0];
        let mut focus = PanelKeyboardFocus {
            menu_anchor: Some(anchor),
            ..PanelKeyboardFocus::default()
        };
        focus.popup_closed(&mut runtime);
        assert_eq!(runtime.focused(), None);
        focus.changed(&mut runtime, true);
        assert_eq!(runtime.focused(), Some(anchor));
        assert!(focus.returning.is_none());
    }
}

fn dispatch_popup_events(
    window: &mut LayerWindow,
    control: &LayerControl,
    ui: &mut Option<LayerPopupUi>,
) -> bool {
    let mut closed = false;
    for event in window.poll_popup_events() {
        if matches!(event, LayerPopupEvent::Closed) {
            *ui = None;
            closed = true;
        } else if let Some(ui) = ui {
            ui.handle_event(&event);
        }
        control.dispatch_popup_event(event);
    }
    closed
}

#[doc = include_str!("../../docs/api/app/layer_runner.doc/install_text.md")]
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
        renderer.register_font(handle, std::sync::Arc::clone(bytes));
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

#[doc = include_str!("../../docs/api/app/layer_runner.doc/init_gpu.md")]
fn init_gpu(
    window: &LayerWindow,
    render_backend: super::RenderBackend,
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
    // Not `Instance::default()`, and the difference is one env var and one
    // backend set. `Instance::default()` builds its flags from
    // `InstanceFlags::from_build_config()` and NEVER consults the environment,
    // so `WGPU_VALIDATION_INDIRECT_CALL=0` is silently ignored, and it asks for
    // `Backends::all()` in one instance. `gpu_instance::open` reads the
    // environment for the FLAGS — with no variable set they are bit-for-bit the
    // ones we had — and asks for the first-tier backends before all of them.
    // `WGPU_BACKEND` it deliberately does not read: `gpu_instance.doc/attempts.md`
    // has what that cost, which is that a backend set the environment can empty
    // is a keyboard that does not start.
    //
    // Why the flag knob is worth having. `from_build_config()` does not return
    // `empty()` in release — its own doc comment says it does and the code
    // disagrees (wgpu-types 29, `instance.rs`): release gets
    // `VALIDATION_INDIRECT_CALL`. wgpu-core then injects, ahead of every
    // `dispatch_workgroups_indirect`, a pipeline swap, two `set_bind_group`s,
    // two buffer barriers and a `dispatch([1, 1, 1])`, then restores the state.
    // Vello issues exactly two indirect dispatches a frame (`path_count`,
    // `path_tiling`), so this is paid twice per present. Measured on an Intel
    // Arc iGPU with two instances in one process, blocks alternating: 13-16 us
    // a frame, three runs, split-half drift under 1%.
    //
    // It is NOT turned off here, and that is deliberate. What the validation
    // clamps are dispatch counts vello computes on the GPU out of its bump
    // allocators; on a scene that overflows those buffers an unclamped indirect
    // dispatch is not a wrong pixel, it is a hung GPU — on a device whose only
    // way of typing is this program. Sixty-two keys will not overflow them, but
    // that is a statement about today's scene, not about this code. So: the cost
    // is named, the measurement exists, and turning it off is now a thing the
    // tablet can be asked to do for one run instead of a thing a rebuild decides.
    //
    // The BACKENDS half is a different animal and it is not a knob, it is a
    // saving: an instance is one `try_add_hal` per backend compiled in, so
    // `Backends::all()` was bringing up the whole Mesa GLES stack next to the
    // Vulkan one and never drawing with it. 58 MB of RSS and four DRM handles,
    // measured; `gpu_instance.doc/_module.md` has the numbers and the fallback
    // that keeps a Vulkan-less machine working.
    let (_instance, surface, adapter) =
        super::gpu_instance::open(|instance| instance.create_surface(window.target()))?;
    let (device, queue) = super::gpu_instance::request_device(&adapter)?;
    let (width, height) = physical_extent(
        window.physical_size(),
        device.limits().max_texture_dimension_2d,
    )?;
    let mut config = surface
        .get_default_config(&adapter, width, height)
        .ok_or_else(|| AppError::Gpu("surface unsupported by adapter".to_owned()))?;
    super::gpu_instance::apply_frame_latency(&mut config);
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

    let renderer = super::gpu_instance::create_renderer(
        render_backend,
        RendererId::from_raw(2),
        device.clone(),
        queue.clone(),
    )?;
    let vello_surface = VelloSurface::new(surface, config);
    vello_surface.configure(&device);
    Ok((renderer, vello_surface, device, queue, adapter.get_info()))
}

/// An informational diagnostic naming the adapter and backend wgpu chose.
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

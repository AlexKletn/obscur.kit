//! Prepare the retained scene independently of the swapchain image.

use std::time::{Duration, Instant};

use crate::scene::scene::SceneSnapshot;

use super::{AppRunner, MAX_FRAME_DELTA};

pub(super) struct PreparedFrame {
    pub(super) snapshot: SceneSnapshot,
    pub(super) built: Duration,
    pub(super) snapshotted: Duration,
    pub(super) work: Duration,
}

impl AppRunner {
    /// Requests a redraw of the live window, if any.
    pub(super) fn request_redraw(&self) {
        #[cfg(target_os = "linux")]
        if self.wayland_frame_blocked() {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            if self.detailed_stats && self.frame_due() {
                println!(
                    "[window-redraw-request] next_id={} at_us={}",
                    self.frame_sequence + 1,
                    self.now_micros()
                );
            }
            window.request_redraw();
        }
    }

    pub(super) fn frame_due(&self) -> bool {
        self.prepared_frame.is_some() || self.runtime.has_pending_frame()
    }

    pub(super) fn prepared_frame_stale(&self) -> bool {
        self.control.has_pending()
            || (self.runtime.has_pending_frame() && !self.runtime.has_pending_timeline_frame())
    }

    pub(super) fn prepare_waiting_frame(&mut self) {
        #[cfg(target_os = "linux")]
        if self.wayland_preparation_blocked() {
            return;
        }
        if self.prepare_ahead
            && self.last_frame.is_some()
            && self.prepared_frame.is_none()
            && self.runtime.has_pending_timeline_frame()
            && !self.control.has_pending()
        {
            self.prepared_frame = self.prepare_frame();
            // A prepared animation frame already owes a present. Start acquiring
            // while Winit waits for the compositor callback for the next redraw.
            if self.prepared_frame.is_some()
                && let Some(acquirer) = self.acquirer.as_mut()
            {
                acquirer.request();
            }
            if self.frame_due() {
                self.request_redraw();
            }
        }
    }

    pub(super) fn prepare_frame(&mut self) -> Option<PreparedFrame> {
        let started = Instant::now();
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
        if let Some(enabled) = self.control.take_background_blur()
            && let Some(window) = self.window.clone()
        {
            self.set_background_blur(&window, enabled);
        }

        if self.runtime.run_due_frame_reactive().is_none() {
            // Nothing was due, but the tick above may have scheduled the *next*
            // animation frame; without asking for it the loop would sleep and the
            // animation would stall on its first idle turn.
            if self.runtime.has_pending_frame() {
                self.request_redraw();
            }
            return None;
        }
        let built = started.elapsed();
        // The one point in the frame where application code may touch the runtime
        // (spec §2.6, §4.12): the frame has committed, so the tree exists and a
        // `test-id` can resolve to a real node, and the next frame has not started, so
        // nothing is mid-propagation. A no-op with nothing queued and every name
        // already bound, which is the steady state.
        self.control.apply(&mut self.runtime, &mut self.css);
        let snapshot_at = Instant::now();
        let snapshot = self.runtime.snapshot();
        let snapshotted = snapshot_at.elapsed();
        Some(PreparedFrame {
            snapshot,
            built,
            snapshotted,
            work: started.elapsed(),
        })
    }
}

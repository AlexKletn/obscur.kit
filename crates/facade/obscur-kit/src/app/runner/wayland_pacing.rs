//! Bound timer-driven Wayland redraws by compositor feedback.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use winit::event_loop::{ActiveEventLoop, ControlFlow};

mod pacing_mode;
mod queue_feedback;

use super::AppRunner;
use pacing_mode::PacingMode;
#[cfg(test)]
use queue_feedback::ADAPTATION_WINDOW;
use queue_feedback::QueueFeedback;

const DEFAULT_MAX_PENDING: usize = 2;
const MAX_ADAPTIVE_PENDING: usize = 3;
const FEEDBACK_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Default)]
pub(super) struct WaylandPacer {
    pending: VecDeque<(u64, Instant)>,
    max_pending: usize,
    adaptive_queue: Option<QueueFeedback>,
    resume_after: Option<u64>,
    next: Option<Instant>,
    mode: PacingMode,
    last_presented: Option<Instant>,
    lead: Duration,
    interval: Duration,
    refresh_millihertz: u32,
}

impl WaylandPacer {
    fn new(refresh_millihertz: Option<u32>, mode: PacingMode, max_pending: usize) -> Self {
        let rate = refresh_millihertz
            .filter(|rate| *rate > 0)
            .unwrap_or(60_000);
        Self {
            interval: Duration::from_nanos(1_000_000_000_000 / u64::from(rate)),
            refresh_millihertz: rate,
            mode,
            max_pending,
            ..Default::default()
        }
    }

    pub(super) fn allows(&self, now: Instant) -> bool {
        self.is_suspended()
            || (self.pending.len() < self.max_pending && self.next.is_none_or(|next| next <= now))
    }

    pub(super) fn committed(
        &mut self,
        id: u64,
        started: Instant,
        committed: Instant,
        preparation: Duration,
    ) {
        if self.is_suspended() {
            return;
        }
        if self.mode != PacingMode::Swapchain {
            self.pending.push_back((id, committed));
        }
        self.next = match self.mode {
            PacingMode::Feedback => None,
            PacingMode::Timer | PacingMode::Swapchain => Some(started + self.interval),
            PacingMode::Display => {
                let measured = committed.saturating_duration_since(started) + preparation;
                self.lead = (self.lead * 3 + measured + Duration::from_millis(1)) / 4;
                let lead = self.lead.clamp(self.interval / 4, self.interval * 3 / 4);
                Some(
                    self.last_presented
                        .map_or(started + self.interval, |shown| {
                            // Skip the slot already used by this submission. Reanchor to
                            // measured presentation rather than accumulating timer drift.
                            let preparation_start =
                                started.checked_sub(preparation).unwrap_or(started);
                            let from =
                                (preparation_start + lead + self.interval / 4).max(committed);
                            next_display_slot(shown, from, self.interval) - lead
                        }),
                )
            }
        };
    }

    pub(super) fn is_suspended(&self) -> bool {
        self.resume_after.is_some()
    }

    fn suspend(&mut self, last_committed: u64) {
        self.resume_after = Some(last_committed);
        self.pending.clear();
        self.next = None;
        self.last_presented = None;
        self.lead = Duration::ZERO;
        if let Some(feedback) = self.adaptive_queue.as_mut() {
            *feedback = QueueFeedback::default();
        }
    }

    fn resume_on_presentation(&mut self, frame: u64) -> bool {
        // Only a frame submitted during fallback proves that callbacks have
        // restored progress. Delayed feedback from the old queue is ignored.
        if self.resume_after.is_some_and(|last| frame > last) {
            self.resume_after = None;
            return true;
        }
        false
    }

    pub(super) fn settled(&mut self, id: u64) -> Option<Instant> {
        let index = self.pending.iter().position(|(frame, _)| *frame == id)?;
        self.pending.remove(index).map(|(_, commit)| commit)
    }

    fn adapt_queue(&mut self, delay: Duration, shown: bool) -> Option<usize> {
        let feedback = self.adaptive_queue.as_mut()?;
        if !feedback.observe(delay, shown, self.interval) {
            return None;
        }
        let depth = match self.max_pending {
            DEFAULT_MAX_PENDING if feedback.late >= 8 && feedback.discarded <= 2 => {
                MAX_ADAPTIVE_PENDING
            }
            MAX_ADAPTIVE_PENDING if feedback.discarded >= 4 || feedback.fast >= 8 => {
                DEFAULT_MAX_PENDING
            }
            _ => self.max_pending,
        };
        *feedback = QueueFeedback::default();
        (depth != std::mem::replace(&mut self.max_pending, depth)).then_some(depth)
    }

    fn next_feedback_poll(&self, now: Instant) -> Option<Instant> {
        self.pending.front().map(|(_, first_commit)| {
            // The feedback proxy has its own queue. Winit can read its events
            // without delivering a window event that triggers another poll.
            (now + self.interval).min(*first_commit + FEEDBACK_TIMEOUT)
        })
    }

    fn wake_at(&self, now: Instant) -> Instant {
        let paced = self.next.unwrap_or(now);
        if self.pending.len() >= self.max_pending {
            self.next_feedback_poll(now).unwrap_or(paced)
        } else {
            paced
        }
    }

    fn feedback_timed_out(&self, now: Instant) -> bool {
        self.pending
            .front()
            .is_some_and(|(_, commit)| now.saturating_duration_since(*commit) >= FEEDBACK_TIMEOUT)
    }

    fn update_refresh(&mut self, rate: Option<u32>) -> bool {
        let Some(rate) = rate.filter(|&rate| rate > 0 && rate != self.refresh_millihertz) else {
            return false;
        };
        self.interval = Duration::from_nanos(1_000_000_000_000 / u64::from(rate));
        self.refresh_millihertz = rate;
        self.last_presented = None;
        if let Some(feedback) = self.adaptive_queue.as_mut() {
            *feedback = QueueFeedback::default();
            self.max_pending = DEFAULT_MAX_PENDING;
        }
        true
    }
}

fn next_display_slot(shown: Instant, after: Instant, interval: Duration) -> Instant {
    if after < shown {
        return shown;
    }
    let elapsed = after.duration_since(shown);
    let remainder = elapsed.as_nanos() % interval.as_nanos();
    after + interval - Duration::from_nanos(remainder as u64)
}

impl AppRunner {
    pub(super) fn init_wayland_feedback(&mut self, window: &std::sync::Arc<winit::window::Window>) {
        let mode = match std::env::var("OBSCUR_WAYLAND_REDRAW_AHEAD").as_deref() {
            Ok("1") => Some(PacingMode::Timer),
            Ok("feedback") => Some(PacingMode::Feedback),
            Ok("display") => Some(PacingMode::Display),
            Ok("swapchain") => Some(PacingMode::Swapchain),
            _ => None,
        };
        if !self.detailed_stats && mode.is_none() {
            return;
        }
        match obscur_wayland_effects::WindowPresentation::new(window.clone()) {
            Ok(Some(presentation)) => {
                self.presentation = Some(presentation);
                if let Some(mode) = mode {
                    let refresh = window
                        .current_monitor()
                        .and_then(|output| output.refresh_rate_millihertz());
                    let requested_depth = std::env::var("OBSCUR_WAYLAND_MAX_PENDING");
                    let adaptive = mode == PacingMode::Feedback
                        && requested_depth.as_deref() == Ok("adaptive");
                    let max_pending = match requested_depth.as_deref() {
                        Ok("3") => 3,
                        _ => DEFAULT_MAX_PENDING,
                    };
                    let mut pacer = WaylandPacer::new(refresh, mode, max_pending);
                    if adaptive {
                        pacer.adaptive_queue = Some(QueueFeedback::default());
                    }
                    if self.detailed_stats {
                        println!(
                            "[wayland-pacer] mode={mode:?} refresh_millihertz={} max_pending={max_pending} adaptive={adaptive}",
                            pacer.refresh_millihertz
                        );
                    }
                    self.wayland_pacer = Some(pacer);
                }
            }
            Ok(None) => eprintln!("[window-performance] presentation feedback unavailable"),
            Err(error) => eprintln!("[window-performance] presentation feedback: {error}"),
        }
    }

    pub(super) fn wayland_frame_blocked(&self) -> bool {
        self.wayland_pacer
            .as_ref()
            .is_some_and(|pacer| !pacer.allows(Instant::now()))
    }

    pub(super) fn poll_wayland_presentation(&mut self) {
        if let Some(presentation) = self.presentation.as_mut() {
            let pacer = &mut self.wayland_pacer;
            let trace = self.detailed_stats;
            let clock = pacer
                .as_ref()
                .filter(|pacer| pacer.mode == PacingMode::Display)
                .and_then(|_| presentation.monotonic_sample());
            self.shown_sample.poll(
                presentation,
                &mut self.runtime,
                self.detailed_stats,
                |frame, timestamp| {
                    let Some(pacer) = pacer.as_mut() else { return };
                    if timestamp.is_some() && pacer.resume_on_presentation(frame) && trace {
                        println!("[wayland-pacer] feedback resumed frame={frame}");
                    }
                    if pacer.is_suspended() {
                        return;
                    }
                    if let (Some((clock, now)), Some(timestamp)) = (clock, timestamp) {
                        let shown = Duration::from_micros(timestamp);
                        let instant = if shown <= clock {
                            now.checked_sub(clock - shown)
                        } else {
                            now.checked_add(shown - clock)
                        };
                        if let Some(instant) = instant
                            && pacer
                                .last_presented
                                .is_none_or(|previous| instant > previous)
                        {
                            pacer.last_presented = Some(instant);
                        }
                    }
                    if let Some(commit) = pacer.settled(frame)
                        && (trace || pacer.adaptive_queue.is_some())
                    {
                        let delay = commit.elapsed();
                        let depth = pacer.adapt_queue(delay, timestamp.is_some());
                        if trace {
                            println!(
                                "[wayland-pacer] feedback id={frame} processed_after_ms={:.3}",
                                delay.as_secs_f64() * 1000.0
                            );
                            if let Some(depth) = depth {
                                println!(
                                    "[wayland-pacer] adaptive max_pending={depth} frame={frame}"
                                );
                            }
                        }
                    }
                },
            );
        }
    }

    pub(super) fn wayland_preparation_blocked(&self) -> bool {
        self.wayland_pacer
            .as_ref()
            .is_some_and(|pacer| pacer.mode == PacingMode::Display && !pacer.allows(Instant::now()))
    }

    pub(super) fn schedule_wayland_redraw(&mut self, event_loop: &ActiveEventLoop) {
        let due = self.frame_due();
        let Some(pacer) = self.wayland_pacer.as_mut() else {
            return;
        };
        if pacer.is_suspended() {
            return;
        }
        let now = Instant::now();
        if pacer.feedback_timed_out(now) {
            if self.detailed_stats {
                let (frame, commit) = pacer.pending[0];
                println!(
                    "[wayland-pacer] feedback timeout id={frame} pending={} age_ms={:.3}; using compositor callbacks",
                    pacer.pending.len(),
                    (now - commit).as_secs_f64() * 1000.0
                );
            }
            pacer.suspend(self.frame_sequence);
            event_loop.set_control_flow(ControlFlow::Wait);
            if due {
                self.request_redraw();
            }
            return;
        }
        let refresh = self
            .window
            .as_ref()
            .and_then(|window| window.current_monitor())
            .and_then(|output| output.refresh_rate_millihertz());
        if pacer.update_refresh(refresh) && self.detailed_stats {
            println!(
                "[wayland-pacer] refresh_millihertz={}",
                pacer.refresh_millihertz
            );
        }
        let feedback_poll = pacer.next_feedback_poll(now);
        if !due {
            event_loop
                .set_control_flow(feedback_poll.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
            return;
        }
        let wake = pacer.wake_at(now);
        if wake > now {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                feedback_poll.map_or(wake, |poll| poll.min(wake)),
            ));
        } else {
            event_loop
                .set_control_flow(feedback_poll.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
            if !self
                .acquirer
                .as_ref()
                .is_some_and(|acquirer| acquirer.is_waiting())
            {
                self.request_redraw();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_slots_snap_to_presentation_and_skip_elapsed_slots() {
        let shown = Instant::now();
        let interval = Duration::from_millis(8);
        assert_eq!(
            next_display_slot(shown, shown + Duration::from_millis(3), interval),
            shown + interval
        );
        assert_eq!(
            next_display_slot(shown, shown + 2 * interval, interval),
            shown + 3 * interval
        );
    }

    #[test]
    fn feedback_only_allows_an_immediate_second_frame() {
        let mut pacer = WaylandPacer::new(Some(120_000), PacingMode::Feedback, DEFAULT_MAX_PENDING);
        let now = Instant::now();
        pacer.committed(1, now, now, Duration::ZERO);
        assert!(pacer.allows(now));
        pacer.committed(2, now, now, Duration::ZERO);
        assert!(!pacer.allows(now));
        let _ = pacer.settled(1);
        assert!(pacer.allows(now));
    }

    #[test]
    fn timed_out_pacer_resumes_after_a_fallback_frame_is_presented() {
        let mut pacer = WaylandPacer::new(Some(120_000), PacingMode::Feedback, 2);
        let start = Instant::now();
        pacer.committed(1, start, start, Duration::ZERO);
        pacer.committed(2, start, start, Duration::ZERO);
        let timed_out = start + FEEDBACK_TIMEOUT;
        assert!(pacer.feedback_timed_out(timed_out));

        pacer.suspend(2);
        assert!(pacer.allows(timed_out));
        assert!(!pacer.resume_on_presentation(2));
        pacer.committed(3, timed_out, timed_out, Duration::ZERO);
        assert!(pacer.pending.is_empty());

        assert!(pacer.resume_on_presentation(3));
        pacer.committed(4, timed_out, timed_out, Duration::ZERO);
        assert_eq!(pacer.settled(4), Some(timed_out));
    }

    #[test]
    fn adaptive_queue_grows_for_late_feedback_and_shrinks_for_discarded_frames() {
        let mut pacer = WaylandPacer::new(Some(120_000), PacingMode::Feedback, DEFAULT_MAX_PENDING);
        pacer.adaptive_queue = Some(QueueFeedback::default());
        let slow = pacer.interval * 3;
        for _ in 1..ADAPTATION_WINDOW {
            assert_eq!(pacer.adapt_queue(slow, true), None);
        }
        assert_eq!(pacer.adapt_queue(slow, true), Some(3));
        for _ in 0..8 {
            assert_eq!(pacer.adapt_queue(slow, true), None);
        }
        for _ in 0..3 {
            assert_eq!(pacer.adapt_queue(slow, false), None);
        }
        assert_eq!(pacer.adapt_queue(slow, false), Some(2));
        for _ in 0..ADAPTATION_WINDOW {
            assert_eq!(pacer.adapt_queue(slow, false), None);
        }
    }

    #[test]
    fn swapchain_mode_keeps_timer_spacing_without_waiting_for_presentation() {
        let mut pacer =
            WaylandPacer::new(Some(120_000), PacingMode::Swapchain, DEFAULT_MAX_PENDING);
        let start = Instant::now();
        let refresh = pacer.interval;
        for id in 0..20 {
            let now = start + id * refresh;
            assert!(pacer.allows(now));
            pacer.committed(u64::from(id), now, now, Duration::ZERO);
            assert!(!pacer.allows(now + refresh / 2));
            assert!(!pacer.feedback_timed_out(now));
        }
        assert!(pacer.pending.is_empty());
    }

    #[test]
    fn three_slots_keep_the_same_timer_spacing() {
        let mut pacer = WaylandPacer::new(Some(120_000), PacingMode::Timer, 3);
        let start = Instant::now();
        let refresh = pacer.interval;
        for (id, offset) in (1..=3).zip(0..3) {
            let now = start + offset * refresh;
            assert!(pacer.allows(now));
            pacer.committed(id, now, now, Duration::ZERO);
            assert!(!pacer.allows(now + refresh / 2));
        }
        assert!(!pacer.allows(start + 3 * refresh));
        assert_eq!(pacer.settled(2), Some(start + refresh));
        assert!(pacer.allows(start + 3 * refresh));
    }

    #[test]
    fn waits_for_refresh_and_feedback_without_accumulating_a_backlog() {
        let mut pacer = WaylandPacer::new(Some(120_000), PacingMode::Timer, DEFAULT_MAX_PENDING);
        let start = Instant::now();
        let refresh = pacer.interval;
        assert!(pacer.allows(start));
        pacer.committed(1, start, start + Duration::from_millis(2), Duration::ZERO);
        assert!(!pacer.allows(start + refresh / 2));
        assert!(pacer.allows(start + refresh));
        pacer.committed(2, start + refresh, start + refresh, Duration::ZERO);
        assert!(!pacer.allows(start + 2 * refresh));
        assert_eq!(pacer.settled(2), Some(start + refresh));
        assert!(pacer.allows(start + 2 * refresh));
        pacer.committed(3, start + 2 * refresh, start + 2 * refresh, Duration::ZERO);
        let timeout = start + Duration::from_millis(2) + FEEDBACK_TIMEOUT;
        assert_eq!(pacer.wake_at(start + 3 * refresh), start + 4 * refresh);
        assert_eq!(pacer.wake_at(timeout - refresh / 2), timeout);
        assert!(pacer.feedback_timed_out(timeout));
        assert!(!pacer.allows(timeout));
    }
}

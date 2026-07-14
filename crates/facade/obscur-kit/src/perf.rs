//! Frame performance measurement for windowed hosts (spec §7.9 telemetry, host
//! side).
//!
//! The runtime never reads a clock (it records structured phase *counts*, not
//! wall time — the host owns timing). [`FrameStats`] is the small reusable
//! aggregator a windowed host feeds two per-frame durations — the **CPU** phase
//! (build → reconcile → style → layout → compose) and the **present** phase
//! (GPU submit + blit) — to get a rolling FPS and timing breakdown it can print
//! or draw, instead of every example hand-rolling the same ring buffer.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// One measured frame: the CPU (layout/compose) and GPU (present) durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSample {
    /// Time spent producing the frame on the CPU (the runtime's reactive frame:
    /// build → reconcile → style → layout → compose).
    pub cpu: Duration,
    /// Time spent presenting the frame on the GPU (submit + blit to surface).
    pub present: Duration,
}

impl FrameSample {
    /// The total wall-clock cost of the frame (CPU + present).
    #[must_use]
    pub fn total(self) -> Duration {
        self.cpu + self.present
    }
}

/// A rolling window of the most recent [`FrameSample`]s, with FPS and timing
/// summaries derived from them.
///
/// The window is a fixed-capacity ring: once full, each new sample evicts the
/// oldest, so the stats always describe the recent past.
#[derive(Debug, Clone)]
pub struct FrameStats {
    samples: VecDeque<FrameSample>,
    capacity: usize,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new(120)
    }
}

impl FrameStats {
    /// Creates a stats window retaining the last `capacity` frames (min `1`).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    /// Records one frame's CPU and present durations, evicting the oldest sample
    /// when the window is full.
    pub fn record(&mut self, cpu: Duration, present: Duration) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(FrameSample { cpu, present });
    }

    /// The number of frames currently in the window.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether no frames have been recorded yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The mean of `f` over the window, or `Duration::ZERO` when empty.
    fn mean(&self, f: impl Fn(&FrameSample) -> Duration) -> Duration {
        let n = self.samples.len();
        if n == 0 {
            return Duration::ZERO;
        }
        let total: Duration = self.samples.iter().map(&f).sum();
        total / n as u32
    }

    /// The mean CPU (layout/compose) time per frame.
    #[must_use]
    pub fn avg_cpu(&self) -> Duration {
        self.mean(|s| s.cpu)
    }

    /// The mean present (GPU) time per frame.
    #[must_use]
    pub fn avg_present(&self) -> Duration {
        self.mean(|s| s.present)
    }

    /// The mean total time per frame.
    #[must_use]
    pub fn avg_total(&self) -> Duration {
        self.mean(|s| s.total())
    }

    /// The worst (largest) total frame time in the window.
    #[must_use]
    pub fn max_total(&self) -> Duration {
        self.samples.iter().map(|s| s.total()).max().unwrap_or(Duration::ZERO)
    }

    /// The sustainable FPS implied by the mean frame cost (`1000 / avg_total_ms`)
    /// — the rate the frame *could* be produced at back-to-back. `0.0` when
    /// empty. For the *actual* present rate under a continuous redraw, divide the
    /// frames recorded by the wall-clock interval instead.
    #[must_use]
    pub fn cost_fps(&self) -> f32 {
        let ms = self.avg_total().as_secs_f32() * 1000.0;
        if ms > 0.0 { 1000.0 / ms } else { 0.0 }
    }

    /// A one-line human summary: `cost fps + cpu/gpu/total ms (+ worst)`.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "~{:.0} fps · cpu {:.2}ms · gpu {:.2}ms · frame {:.2}ms (max {:.2}ms) · {} frames",
            self.cost_fps(),
            ms(self.avg_cpu()),
            ms(self.avg_present()),
            ms(self.avg_total()),
            ms(self.max_total()),
            self.len(),
        )
    }

    /// Empties the window.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// A [`Duration`] as fractional milliseconds.
fn ms(d: Duration) -> f32 {
    d.as_secs_f32() * 1000.0
}

/// A drop-in per-frame profiler a windowed host feeds two measured durations —
/// the CPU frame and the GPU present — to get a rolling [`FrameStats`] plus a
/// ready-to-print report line at a fixed cadence.
///
/// This is the whole "measure FPS + timings" facility as *kit* code: a host
/// records each frame's two spans and prints the returned line; it no longer
/// hand-rolls a ring buffer, an interval timer, and a format string per example.
/// The real present FPS is `frames this interval / wall-clock interval`, so the
/// host should request a redraw every frame for a continuous rate.
#[derive(Debug, Clone)]
pub struct FrameProfiler {
    stats: FrameStats,
    interval: Duration,
    report_start: Option<Instant>,
    frames_this_interval: u32,
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameProfiler {
    /// Creates a profiler reporting once per second over a 120-frame window.
    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(Duration::from_secs(1))
    }

    /// Creates a profiler reporting once per `interval`.
    #[must_use]
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            stats: FrameStats::default(),
            interval,
            report_start: None,
            frames_this_interval: 0,
        }
    }

    /// Records one frame's CPU and present durations. Returns a formatted
    /// `"N fps · cpu …ms · gpu …ms · frame …ms (worst …ms)"` line **once per
    /// interval** (the wall-clock present FPS plus the rolling timing breakdown),
    /// or `None` on the frames in between.
    pub fn record(&mut self, cpu: Duration, present: Duration) -> Option<String> {
        self.stats.record(cpu, present);
        self.frames_this_interval += 1;
        let start = *self.report_start.get_or_insert_with(Instant::now);
        let elapsed = start.elapsed();
        if elapsed < self.interval {
            return None;
        }
        let fps = self.frames_this_interval as f32 / elapsed.as_secs_f32();
        let line = format!(
            "{fps:.0} fps · cpu {:.2}ms · gpu {:.2}ms · frame {:.2}ms (worst {:.2}ms)",
            ms(self.stats.avg_cpu()),
            ms(self.stats.avg_present()),
            ms(self.stats.avg_total()),
            ms(self.stats.max_total()),
        );
        self.report_start = Some(Instant::now());
        self.frames_this_interval = 0;
        Some(line)
    }

    /// The underlying rolling stats (e.g. to build a DevTools `Rendering`
    /// snapshot with [`crate::devtools::perf_payload`]).
    #[must_use]
    pub fn stats(&self) -> &FrameStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dur(us: u64) -> Duration {
        Duration::from_micros(us)
    }

    #[test]
    fn averages_over_the_window() {
        let mut stats = FrameStats::new(8);
        stats.record(dur(2000), dur(6000)); // 2ms cpu + 6ms gpu = 8ms
        stats.record(dur(4000), dur(4000)); // 4 + 4 = 8ms
        assert_eq!(stats.len(), 2);
        assert_eq!(stats.avg_cpu(), dur(3000));
        assert_eq!(stats.avg_present(), dur(5000));
        assert_eq!(stats.avg_total(), dur(8000));
        assert_eq!(stats.max_total(), dur(8000));
        // 8ms/frame → ~125 fps sustainable.
        assert!((stats.cost_fps() - 125.0).abs() < 0.5, "{}", stats.cost_fps());
    }

    #[test]
    fn ring_evicts_the_oldest_when_full() {
        let mut stats = FrameStats::new(2);
        stats.record(dur(1000), dur(1000)); // total 2ms
        stats.record(dur(1000), dur(1000));
        stats.record(dur(9000), dur(9000)); // evicts the first → window is [2ms, 18ms]
        assert_eq!(stats.len(), 2);
        assert_eq!(stats.avg_total(), dur(10000));
        assert_eq!(stats.max_total(), dur(18000));
    }

    #[test]
    fn profiler_reports_once_per_interval() {
        // A zero interval reports on every frame; the line names fps + timings.
        let mut p = FrameProfiler::with_interval(Duration::ZERO);
        let line = p.record(dur(2000), dur(6000)).expect("zero interval reports immediately");
        assert!(line.contains("fps"), "{line}");
        assert!(line.contains("gpu 6.00ms"), "{line}");
        assert_eq!(p.stats().len(), 1);
    }

    #[test]
    fn profiler_holds_reports_until_the_interval_elapses() {
        // A one-hour interval never fires in a test, but the frame is still
        // recorded into the rolling stats.
        let mut p = FrameProfiler::with_interval(Duration::from_secs(3600));
        assert!(p.record(dur(1000), dur(1000)).is_none());
        assert!(p.record(dur(1000), dur(1000)).is_none());
        assert_eq!(p.stats().len(), 2);
    }

    #[test]
    fn empty_stats_are_zero_not_nan() {
        let stats = FrameStats::default();
        assert!(stats.is_empty());
        assert_eq!(stats.avg_total(), Duration::ZERO);
        assert_eq!(stats.cost_fps(), 0.0);
    }
}

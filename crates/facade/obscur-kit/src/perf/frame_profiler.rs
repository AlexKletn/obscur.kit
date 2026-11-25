use std::time::{Duration, Instant};

use super::{FrameStats, ms};

#[doc = include_str!("../../docs/api/perf.doc/FrameProfiler.md")]
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

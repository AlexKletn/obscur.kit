#![doc = include_str!("perf.doc/_module.md")]

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[doc = include_str!("perf.doc/FrameSample.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSample {
    #[doc = include_str!("perf.doc/cpu.md")]
    pub cpu: Duration,
    #[doc = include_str!("perf.doc/present.md")]
    pub present: Duration,
}

impl FrameSample {
    #[doc = include_str!("perf.doc/total.md")]
    #[must_use]
    pub fn total(self) -> Duration {
        self.cpu + self.present
    }
}

#[doc = include_str!("perf.doc/FrameStats.md")]
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
    #[doc = include_str!("perf.doc/new.md")]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    #[doc = include_str!("perf.doc/record.md")]
    pub fn record(&mut self, cpu: Duration, present: Duration) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(FrameSample { cpu, present });
    }

    #[doc = include_str!("perf.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    #[doc = include_str!("perf.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[doc = include_str!("perf.doc/mean.md")]
    fn mean(&self, f: impl Fn(&FrameSample) -> Duration) -> Duration {
        let n = self.samples.len();
        if n == 0 {
            return Duration::ZERO;
        }
        let total: Duration = self.samples.iter().map(&f).sum();
        total / n as u32
    }

    #[doc = include_str!("perf.doc/avg_cpu.md")]
    #[must_use]
    pub fn avg_cpu(&self) -> Duration {
        self.mean(|s| s.cpu)
    }

    #[doc = include_str!("perf.doc/avg_present.md")]
    #[must_use]
    pub fn avg_present(&self) -> Duration {
        self.mean(|s| s.present)
    }

    #[doc = include_str!("perf.doc/avg_total.md")]
    #[must_use]
    pub fn avg_total(&self) -> Duration {
        self.mean(|s| s.total())
    }

    #[doc = include_str!("perf.doc/max_total.md")]
    #[must_use]
    pub fn max_total(&self) -> Duration {
        self.samples
            .iter()
            .map(|s| s.total())
            .max()
            .unwrap_or(Duration::ZERO)
    }

    #[doc = include_str!("perf.doc/cost_fps.md")]
    #[must_use]
    pub fn cost_fps(&self) -> f32 {
        let ms = self.avg_total().as_secs_f32() * 1000.0;
        if ms > 0.0 { 1000.0 / ms } else { 0.0 }
    }

    #[doc = include_str!("perf.doc/summary.md")]
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

    #[doc = include_str!("perf.doc/clear.md")]
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

#[doc = include_str!("perf.doc/ms.md")]
fn ms(d: Duration) -> f32 {
    d.as_secs_f32() * 1000.0
}

#[doc = include_str!("perf.doc/FrameProfiler.md")]
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
    #[doc = include_str!("perf.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(Duration::from_secs(1))
    }

    #[doc = include_str!("perf.doc/with_interval.md")]
    #[must_use]
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            stats: FrameStats::default(),
            interval,
            report_start: None,
            frames_this_interval: 0,
        }
    }

    #[doc = include_str!("perf.doc/record_2.md")]
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

    #[doc = include_str!("perf.doc/stats.md")]
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
        assert!(
            (stats.cost_fps() - 125.0).abs() < 0.5,
            "{}",
            stats.cost_fps()
        );
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
        let line = p
            .record(dur(2000), dur(6000))
            .expect("zero interval reports immediately");
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

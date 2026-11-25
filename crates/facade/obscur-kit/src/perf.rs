#![doc = include_str!("../docs/api/perf.doc/_module.md")]

mod frame_profiler;
pub use frame_profiler::FrameProfiler;
mod frame_sample;
pub use frame_sample::FrameSample;
mod frame_stats;
pub use frame_stats::FrameStats;

/// A [`std::time::Duration`] as fractional milliseconds.
fn ms(d: std::time::Duration) -> f32 {
    d.as_secs_f32() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

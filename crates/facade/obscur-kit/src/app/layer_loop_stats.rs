#![doc = include_str!("layer_loop_stats.doc/_module.md")]

use std::time::Duration;

#[doc = include_str!("layer_loop_stats.doc/REPORT_EVERY.md")]
const REPORT_EVERY: u32 = 60;

#[doc = include_str!("layer_loop_stats.doc/LayerLoopStats.md")]
#[derive(Debug, Default)]
pub(super) struct LayerLoopStats {
    #[doc = include_str!("layer_loop_stats.doc/each.md")]
    each: bool,
    turns: u32,
    waited: Duration,
    build: Duration,
    present: Duration,
    worst_present: Duration,
}

impl LayerLoopStats {
    #[doc = include_str!("layer_loop_stats.doc/enabled.md")]
    pub(super) fn enabled() -> Option<Self> {
        std::env::var_os("OBSCUR_FRAME_STATS").map(|asked| Self {
            each: asked.to_string_lossy() == "each",
            ..Self::default()
        })
    }

    #[doc = include_str!("layer_loop_stats.doc/record.md")]
    pub(super) fn record(&mut self, waited: Duration, build: Duration, present: Duration) {
        self.turns += 1;
        self.waited += waited;
        self.build += build;
        self.present += present;
        self.worst_present = self.worst_present.max(present);
        // One line per turn under `OBSCUR_FRAME_STATS=each`: the mean below is
        // for watching, this is for counting. A turn that waited 200 ms and one
        // that waited none average to a loop nobody would look twice at.
        if self.each {
            let ms = |d: Duration| d.as_secs_f64() * 1000.0;
            println!(
                "[turn] waited {:.3} | build {:.3} | present {:.3}",
                ms(waited),
                ms(build),
                ms(present),
            );
        }
        if self.turns < REPORT_EVERY {
            return;
        }
        let n = f64::from(self.turns);
        let ms = |total: Duration| total.as_secs_f64() * 1000.0 / n;
        println!(
            "[layer-loop] {n:.0} turns | waited {:.2}ms | build {:.2}ms | present {:.2}ms \
             (worst {:.2}ms) | turn {:.2}ms",
            ms(self.waited),
            ms(self.build),
            ms(self.present),
            self.worst_present.as_secs_f64() * 1000.0,
            ms(self.waited) + ms(self.build) + ms(self.present),
        );
        *self = Self {
            each: self.each,
            ..Self::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_recorded_until_the_environment_asks() {
        // Off by default: this prints, and a panel that printed sixty lines a
        // second unasked would be worse than no instrument at all.
        //
        // `enabled` reads the process environment, so this asserts the shape it
        // returns rather than mutating it out from under a parallel test.
        let stats = LayerLoopStats::default();
        assert_eq!(stats.turns, 0);
        assert_eq!(stats.present, Duration::ZERO);
    }

    #[test]
    fn the_three_phases_are_kept_apart() {
        // The whole reason this exists rather than reusing `perf::FrameStats`,
        // which holds two numbers: the finding it was written for is that one
        // of the three is hundreds of milliseconds while the others are
        // fractions, and a sum would hide exactly that.
        let mut stats = LayerLoopStats::default();
        stats.record(
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(400),
        );
        assert_eq!(stats.waited, Duration::from_millis(1));
        assert_eq!(stats.build, Duration::from_millis(2));
        assert_eq!(stats.present, Duration::from_millis(400));
    }

    #[test]
    fn the_worst_present_is_kept_because_the_average_hides_a_burst() {
        // 1-D was a burst: most turns were fine and a few were catastrophic.
        // An average over sixty turns reports 7ms for that and reads as healthy.
        let mut stats = LayerLoopStats::default();
        for _ in 0..10 {
            stats.record(Duration::ZERO, Duration::ZERO, Duration::from_millis(1));
        }
        stats.record(Duration::ZERO, Duration::ZERO, Duration::from_millis(500));
        assert_eq!(stats.worst_present, Duration::from_millis(500));
        assert!(
            stats.present.as_secs_f64() * 1000.0 / f64::from(stats.turns) < 50.0,
            "the average alone would not have shown it"
        );
    }

    #[test]
    fn the_counters_reset_after_a_report() {
        let mut stats = LayerLoopStats::default();
        for _ in 0..REPORT_EVERY {
            stats.record(
                Duration::from_millis(1),
                Duration::from_millis(1),
                Duration::from_millis(1),
            );
        }
        assert_eq!(stats.turns, 0, "the window starts again");
        assert_eq!(stats.worst_present, Duration::ZERO);
    }
}

//! Summary of application frame work over a fixed sample.

#[doc = include_str!("../../docs/api/app/frame_stats.doc/FrameStats.md")]
#[derive(Default)]
pub(super) struct FrameStats {
    frames: u32,
    build: std::time::Duration,
    snapshot: std::time::Duration,
    present: std::time::Duration,
    worst: std::time::Duration,
}

/// How many presented frames each stats line covers.
const REPORT_EVERY: u32 = 60;

impl FrameStats {
    /// Folds one presented frame in, printing a summary every [`REPORT_EVERY`].
    pub(super) fn record(
        &mut self,
        build: std::time::Duration,
        snapshot: std::time::Duration,
        present: std::time::Duration,
    ) {
        self.frames += 1;
        self.build += build;
        self.snapshot += snapshot;
        self.present += present;
        self.worst = self.worst.max(build + snapshot + present);
        if self.frames < REPORT_EVERY {
            return;
        }
        let n = f64::from(self.frames);
        let ms = |total: std::time::Duration| total.as_secs_f64() * 1000.0 / n;
        let total = ms(self.build) + ms(self.snapshot) + ms(self.present);
        println!(
            "[frame-stats] {n:.0} frames | build {:.2}ms | snapshot {:.2}ms | present {:.2}ms | \
             total {total:.2}ms (~{:.0} fps if saturated) | worst {:.2}ms",
            ms(self.build),
            ms(self.snapshot),
            ms(self.present),
            1000.0 / total.max(0.001),
            self.worst.as_secs_f64() * 1000.0,
        );
        *self = Self::default();
    }
}

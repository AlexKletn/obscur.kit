#![doc = include_str!("frame_phase_clock.doc/_module.md")]

#[doc = include_str!("frame_phase_clock.doc/FramePhaseClock.md")]
pub(crate) struct FramePhaseClock {
    enabled: bool,
    #[doc = include_str!("frame_phase_clock.doc/began.md")]
    began: std::time::Instant,
    last: std::cell::Cell<std::time::Instant>,
    reconcile: std::cell::Cell<std::time::Duration>,
    style: std::cell::Cell<std::time::Duration>,
    layout: std::cell::Cell<std::time::Duration>,
}

#[doc = include_str!("frame_phase_clock.doc/PHASE_REPORT_EVERY.md")]
const PHASE_REPORT_EVERY: u32 = 60;

thread_local! {
    #[doc = include_str!("frame_phase_clock.doc/PHASE_TOTALS.md")]
    static PHASE_TOTALS: std::cell::RefCell<(u32, std::time::Duration, std::time::Duration, std::time::Duration, std::time::Duration, std::time::Duration)> =
        const { std::cell::RefCell::new((0, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO)) };
}

impl FramePhaseClock {
    pub(crate) fn start() -> Self {
        let now = std::time::Instant::now();
        Self {
            enabled: std::env::var_os("OBSCUR_FRAME_STATS").is_some(),
            began: now,
            last: std::cell::Cell::new(now),
            reconcile: std::cell::Cell::new(std::time::Duration::ZERO),
            style: std::cell::Cell::new(std::time::Duration::ZERO),
            layout: std::cell::Cell::new(std::time::Duration::ZERO),
        }
    }

    #[doc = include_str!("frame_phase_clock.doc/lap.md")]
    fn lap(&self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last.get());
        self.last.set(now);
        elapsed
    }

    pub(crate) fn mark_reconcile(&self) {
        if self.enabled {
            self.reconcile.set(self.lap());
        }
    }

    pub(crate) fn mark_style(&self) {
        if self.enabled {
            self.style.set(self.lap());
        }
    }

    pub(crate) fn mark_layout(&self) {
        if self.enabled {
            self.layout.set(self.lap());
        }
    }

    #[doc = include_str!("frame_phase_clock.doc/finish.md")]
    pub(crate) fn finish(&self) {
        if !self.enabled {
            return;
        }
        let compose = self.lap();
        let total = self.began.elapsed();
        let (reconcile, style, layout) =
            (self.reconcile.get(), self.style.get(), self.layout.get());
        PHASE_TOTALS.with(|totals| {
            let mut t = totals.borrow_mut();
            t.0 += 1;
            t.1 += reconcile;
            t.2 += style;
            t.3 += layout;
            t.4 += compose;
            t.5 += total;
            if t.0 < PHASE_REPORT_EVERY {
                return;
            }
            let n = f64::from(t.0);
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / n;
            // Everything the four marks do not bracket: input dispatch, generated
            // content, the editing reseed, accessibility, snapshot install.
            let other = (ms(t.5) - ms(t.1) - ms(t.2) - ms(t.3) - ms(t.4)).max(0.0);
            println!(
                "[phase-stats] {n:.0} frames | reconcile+build {:.2}ms | style {:.2}ms | layout {:.2}ms | compose {:.2}ms | other {other:.2}ms | frame {:.2}ms",
                ms(t.1),
                ms(t.2),
                ms(t.3),
                ms(t.4),
                ms(t.5),
            );
            *t = (0, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO);
        });
    }
}

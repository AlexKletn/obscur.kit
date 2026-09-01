#![doc = include_str!("frame_phase_clock.doc/_module.md")]

#[doc = include_str!("frame_phase_clock.doc/FramePhaseClock.md")]
pub(crate) struct FramePhaseClock {
    enabled: bool,
    #[doc = include_str!("frame_phase_clock.doc/each.md")]
    each: bool,
    #[doc = include_str!("frame_phase_clock.doc/began.md")]
    began: std::time::Instant,
    last: std::cell::Cell<std::time::Instant>,
    reconcile: std::cell::Cell<std::time::Duration>,
    style: std::cell::Cell<std::time::Duration>,
    layout: std::cell::Cell<std::time::Duration>,
    #[doc = include_str!("frame_phase_clock.doc/styled.md")]
    styled: std::cell::Cell<(u64, u64)>,
}

#[doc = include_str!("frame_phase_clock.doc/PHASE_REPORT_EVERY.md")]
const PHASE_REPORT_EVERY: u32 = 60;

#[doc = include_str!("frame_phase_clock.doc/PhaseTotals.md")]
#[derive(Default, Clone, Copy)]
struct PhaseTotals {
    builds: u32,
    reconcile: std::time::Duration,
    style: std::time::Duration,
    layout: std::time::Duration,
    compose: std::time::Duration,
    build: std::time::Duration,
    resolved: u64,
    nodes: u64,
}

thread_local! {
    #[doc = include_str!("frame_phase_clock.doc/PHASE_TOTALS.md")]
    static PHASE_TOTALS: std::cell::RefCell<PhaseTotals> =
        const {
            std::cell::RefCell::new(PhaseTotals {
                builds: 0,
                reconcile: std::time::Duration::ZERO,
                style: std::time::Duration::ZERO,
                layout: std::time::Duration::ZERO,
                compose: std::time::Duration::ZERO,
                build: std::time::Duration::ZERO,
                resolved: 0,
                nodes: 0,
            })
        };
}

impl FramePhaseClock {
    pub(crate) fn start() -> Self {
        let now = std::time::Instant::now();
        Self {
            enabled: std::env::var_os("OBSCUR_FRAME_STATS").is_some(),
            each: std::env::var_os("OBSCUR_FRAME_STATS")
                .is_some_and(|asked| asked.to_string_lossy() == "each"),
            began: now,
            last: std::cell::Cell::new(now),
            reconcile: std::cell::Cell::new(std::time::Duration::ZERO),
            style: std::cell::Cell::new(std::time::Duration::ZERO),
            layout: std::cell::Cell::new(std::time::Duration::ZERO),
            styled: std::cell::Cell::new((0, 0)),
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

    #[doc = include_str!("frame_phase_clock.doc/record_style_scope.md")]
    pub(crate) fn record_style_scope(&self, resolved: u64, nodes: u64) {
        if self.enabled {
            self.styled.set((resolved, nodes));
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
        // `OBSCUR_FRAME_STATS=each` prints every frame instead of a mean over
        // sixty. The mean is what hid the fault it was meant to find: a window
        // averaging 17 ms held a frame of 234, and the two read identically.
        // One line per frame is more output than anyone wants to watch and
        // exactly what a distribution needs.
        if self.each {
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            println!(
                "[frame] reconcile+build {:.3} | style {:.3} | layout {:.3} | compose {:.3} | build {:.3}",
                ms(reconcile),
                ms(style),
                ms(layout),
                ms(compose),
                ms(total),
            );
        }
        PHASE_TOTALS.with(|totals| {
            let mut t = totals.borrow_mut();
            t.builds += 1;
            t.reconcile += reconcile;
            t.style += style;
            t.layout += layout;
            t.compose += compose;
            t.build += total;
            let (resolved, nodes) = self.styled.get();
            t.resolved += resolved;
            t.nodes += nodes;
            if t.builds < PHASE_REPORT_EVERY {
                return;
            }
            let n = f64::from(t.builds);
            #[allow(clippy::cast_precision_loss)]
            let (resolved_per_frame, nodes_per_frame) = (t.resolved as f64 / n, t.nodes as f64 / n);
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / n;
            // Everything the four marks do not bracket: input dispatch, generated
            // content, the editing reseed, accessibility, snapshot install.
            let other =
                (ms(t.build) - ms(t.reconcile) - ms(t.style) - ms(t.layout) - ms(t.compose))
                    .max(0.0);
            println!(
                // `build`, not `frame`. What is bracketed here ends when
                // `run_due_frame_reactive` returns: rasterising, acquiring a
                // swapchain image, blitting, submitting and presenting all
                // happen after that, and none of them is in this number. A
                // subtotal called `frame` sent a reader hunting through `style`
                // for time that was being spent in `present` (audit 3-D).
                // `styled` is the question `style {:.2}ms` cannot answer: a
                // phase that costs milliseconds because it resolved every node
                // and one that costs them because each node is expensive need
                // opposite fixes, and the number telling them apart was only
                // ever visible through a telemetry sink nobody installs.
                "[phase-stats] {n:.0} builds | reconcile+build {:.2}ms | style {:.2}ms | layout {:.2}ms | compose {:.2}ms | other {other:.2}ms | build {:.2}ms | styled {:.1}/{:.1} nodes (present is NOT in this)",
                ms(t.reconcile),
                ms(t.style),
                ms(t.layout),
                ms(t.compose),
                ms(t.build),
                resolved_per_frame,
                nodes_per_frame,
            );
            *t = PhaseTotals::default();
        });
    }
}

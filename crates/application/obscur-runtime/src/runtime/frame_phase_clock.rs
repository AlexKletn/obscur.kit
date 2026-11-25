//! Per-phase wall-clock profiling of one frame: [`FramePhaseClock`].

use super::phase_totals::PhaseTotals;

#[doc = include_str!("../../docs/api/runtime/frame_phase_clock.doc/FramePhaseClock.md")]
pub(crate) struct FramePhaseClock {
    enabled: bool,
    #[doc = include_str!("../../docs/api/runtime/frame_phase_clock.doc/each.md")]
    each: bool,
    /// When the frame began — the first statement of `run_frame` — so the phases can
    /// be reconciled against the whole of it, and what lies outside them can be seen.
    began: std::time::Instant,
    last: std::cell::Cell<std::time::Instant>,
    reconcile: std::cell::Cell<std::time::Duration>,
    style: std::cell::Cell<std::time::Duration>,
    layout: std::cell::Cell<std::time::Duration>,
    // Accumulated, not a single lap: composition is paid in two places — the
    // image/vector resolution that runs before layout (it is compose's work,
    // and used to be billed to `style`) and the display-list build itself.
    compose: std::cell::Cell<std::time::Duration>,
    /// How many nodes the style phase resolved this frame, and how many the tree holds.
    styled: std::cell::Cell<(u64, u64)>,
    // How many mounted components were still stale when the frame ended, i.e.
    // how many of them made this frame ask for another one. A loop that turns
    // forever with nothing on screen changing shows up here and nowhere else.
    stale: std::cell::Cell<usize>,
}

/// How many frames each `[phase-stats]` line covers.
const PHASE_REPORT_EVERY: u32 = 60;

thread_local! {
    /// Rolling totals across frames, printed every [`PHASE_REPORT_EVERY`].
    static PHASE_TOTALS: std::cell::RefCell<PhaseTotals> =
        const { std::cell::RefCell::new(PhaseTotals::ZERO) };

    /// What the host loop paid to put the PREVIOUS frame on the screen.
    ///
    /// Rasterising, acquiring the swapchain image, blitting, submitting and
    /// committing all happen after `run_frame` returns, so a frame cannot time
    /// its own present — by the time this frame's phases are reported, this
    /// frame has not been presented yet. The host hands the number back through
    /// [`record_present_time`] and the next frame prints it, shifted by exactly
    /// one turn. Over a run of hundreds of frames that shift is invisible in a
    /// distribution and it is the only honest way to get the number into the
    /// same line as the phases; the unshifted per-turn value stays available as
    /// `[turn] ... present`.
    ///
    /// A `Cell` on the frame thread, not an atomic: the layer loop builds and
    /// presents on one thread, so this is a plain store with no contention and
    /// no lock. A host that presents on another thread simply never writes it,
    /// and the field reads zero — which is why the report line says
    /// `present` is the loop's number, not the frame's.
    ///
    /// **Read once, then cleared.** Each present belongs to exactly one frame —
    /// the next one — and is reported for that frame and no other. Peeked
    /// rather than taken, as this was, a turn that built a frame without
    /// presenting it (the render failed, the host is windowed and reports
    /// nothing at all, the loop reached `finish` without reaching the commit)
    /// carried the *previous* turn's present into its own line, and
    /// `turn = build + present` for that frame was a sum over a number
    /// belonging to a different frame. Repeat it over a stretch where presents
    /// are rarer than builds and the mean is inflated by exactly the ratio.
    /// A zero now means what it says: no present was reported for this frame.
    /// See [`take_present_time`].
    static LAST_PRESENT: std::cell::Cell<std::time::Duration> =
        const { std::cell::Cell::new(std::time::Duration::ZERO) };
}

/// Whether `OBSCUR_FRAME_STATS` asked for stats, and whether it asked for a
/// line per frame — read once per process instead of twice per frame.
///
/// `FramePhaseClock::start()` runs on every frame and used to call
/// `std::env::var_os` twice from it. Reading the environment is a lock plus a
/// scan of `environ`, paid whether or not anyone asked for stats: an instrument
/// charging the un-instrumented path. The answer cannot change during the
/// process, so it is settled once.
fn asked() -> (bool, bool) {
    static ASKED: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
    *ASKED.get_or_init(|| {
        let asked = std::env::var_os("OBSCUR_FRAME_STATS");
        let each = asked
            .as_ref()
            .is_some_and(|value| value.to_string_lossy() == "each");
        (asked.is_some(), each)
    })
}

/// Tell the phase clock what the last present cost, so the next frame's line
/// can carry it. See `LAST_PRESENT`.
///
/// Called by the host loop from the thread that builds frames, and only while
/// it is keeping stats — off that path this is never called and costs nothing.
pub fn record_present_time(took: std::time::Duration) {
    LAST_PRESENT.with(|cell| cell.set(took));
}

/// The present waiting to be reported, and nothing afterwards.
///
/// Taking rather than peeking is the whole of the fix described on
/// `LAST_PRESENT`: one present, one frame's line. A frame with nothing waiting
/// reads `ZERO`, and the report says so rather than repeating a neighbour's
/// number.
fn take_present_time() -> std::time::Duration {
    LAST_PRESENT.with(|cell| cell.replace(std::time::Duration::ZERO))
}

impl FramePhaseClock {
    pub(crate) fn detailed(&self) -> bool {
        self.each
    }

    pub(crate) fn start() -> Self {
        let now = std::time::Instant::now();
        let (enabled, each) = asked();
        Self {
            enabled,
            each,
            began: now,
            last: std::cell::Cell::new(now),
            reconcile: std::cell::Cell::new(std::time::Duration::ZERO),
            style: std::cell::Cell::new(std::time::Duration::ZERO),
            layout: std::cell::Cell::new(std::time::Duration::ZERO),
            compose: std::cell::Cell::new(std::time::Duration::ZERO),
            styled: std::cell::Cell::new((0, 0)),
            stale: std::cell::Cell::new(0),
        }
    }

    /// Closes the current phase and returns its duration.
    fn lap(&self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last.get());
        self.last.set(now);
        elapsed
    }

    /// End the current stretch and bill it to no phase.
    ///
    /// What lies between `start()` and the first phase — input dispatch, the
    /// resource commit, choosing the plan — is real frame time that belongs to
    /// none of the four phases. Discarding the lap here is what lets it show up
    /// in `other`, which is the whole point: `other` used to be zero by
    /// construction because the four marks tiled the entire measured interval,
    /// so the one number that could have said "the frame is somewhere you are
    /// not looking" could only ever say zero.
    pub(crate) fn unattributed(&self) {
        if self.enabled {
            self.lap();
        }
    }

    pub(crate) fn mark_reconcile(&self) {
        if self.enabled {
            self.reconcile.set(self.lap());
        }
    }

    #[doc = include_str!("../../docs/api/runtime/frame_phase_clock.doc/record_style_scope.md")]
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

    /// Resolving this frame's images and vectors: compose's work, done before
    /// layout because a replaced element is measured against its content.
    pub(crate) fn mark_images(&self) {
        if self.enabled {
            self.compose.set(self.compose.get() + self.lap());
        }
    }

    pub(crate) fn mark_layout(&self) {
        if self.enabled {
            self.layout.set(self.lap());
        }
    }

    /// The display-list build. Added to whatever `mark_images` already billed.
    pub(crate) fn mark_compose(&self) {
        if self.enabled {
            self.compose.set(self.compose.get() + self.lap());
        }
    }

    /// How many components are still stale as the frame ends — the count that
    /// says whether the loop is about to ask for another frame, and why.
    ///
    /// Takes a closure because counting them walks the mounted components, and
    /// an instrument nobody switched on must not walk anything.
    pub(crate) fn note_stale(&self, count: impl FnOnce() -> usize) {
        if self.enabled {
            self.stale.set(count());
        }
    }

    #[doc = include_str!("../../docs/api/runtime/frame_phase_clock.doc/finish.md")]
    pub(crate) fn finish(&self) {
        if !self.enabled {
            return;
        }
        let total = self.began.elapsed();
        let (reconcile, style, layout, compose) = (
            self.reconcile.get(),
            self.style.get(),
            self.layout.get(),
            self.compose.get(),
        );
        // Saturating, because the four phases are now a strict SUBSET of the
        // measured interval rather than a tiling of it: they cannot exceed it,
        // and if clock skew ever made them appear to, a negative `other` would
        // be a worse lie than a zero.
        let other = total
            .saturating_sub(reconcile)
            .saturating_sub(style)
            .saturating_sub(layout)
            .saturating_sub(compose);
        // Taken, not peeked: this present belongs to this frame's line and to
        // no other. See `LAST_PRESENT`.
        let present = take_present_time();
        let stale = self.stale.get();
        // `OBSCUR_FRAME_STATS=each` prints every frame instead of a mean over
        // sixty. The mean is what hid the fault it was meant to find: a window
        // averaging 17 ms held a frame of 234, and the two read identically.
        // One line per frame is more output than anyone wants to watch and
        // exactly what a distribution needs.
        if self.each {
            let (resolved, nodes) = self.styled.get();
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            println!(
                "[frame] reconcile+build {:.3} | style {:.3} | layout {:.3} | compose {:.3} \
                 | other {:.3} | build {:.3} | present {:.3} | stale {stale} | styled {resolved}/{nodes}",
                ms(reconcile),
                ms(style),
                ms(layout),
                ms(compose),
                ms(other),
                ms(total),
                ms(present),
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
            t.present += present;
            // Counted apart from `builds`, because now that a present is taken
            // rather than peeked the two are no longer the same number: a frame
            // with nothing waiting contributes a zero, and averaging those
            // zeros over every build would report a present cheaper than any
            // present ever was. Dividing by the frames that carried one keeps
            // `present` the cost of a present; `builds - presents` is then a
            // fact worth seeing in its own right — frames built that nobody
            // showed.
            t.presents += u32::from(!present.is_zero());
            let (resolved, nodes) = self.styled.get();
            t.resolved += resolved;
            t.nodes += nodes;
            t.stale += stale as u64;
            t.stale_frames += u32::from(stale > 0);
            if t.builds < PHASE_REPORT_EVERY {
                return;
            }
            let n = f64::from(t.builds);
            #[allow(clippy::cast_precision_loss)]
            let (resolved_per_frame, nodes_per_frame) = (t.resolved as f64 / n, t.nodes as f64 / n);
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / n;
            // Over the presents, not over the builds — see `t.presents`.
            let present_ms = if t.presents == 0 {
                0.0
            } else {
                t.present.as_secs_f64() * 1000.0 / f64::from(t.presents)
            };
            // `other` is now measured, not derived from a tiling: `start()` sits
            // at the top of `run_frame` and `finish()` at the bottom, so input
            // dispatch, the resource commit, the plan, the snapshot, the
            // accessibility publish and the scroll refresh are all inside the
            // bracket and all outside the four phases. When it is large, the
            // frame is somewhere none of the four phases looks.
            let other = ms(t.build) - ms(t.reconcile) - ms(t.style) - ms(t.layout) - ms(t.compose);
            #[allow(clippy::cast_precision_loss)]
            let stale_per_frame = t.stale as f64 / n;
            println!(
                // `build` is still the build: everything `run_frame` does.
                // `present` is the host loop's own number for the PREVIOUS
                // turn — rasterise, acquire, blit, submit, commit — handed back
                // through `record_present_time`, and `turn` is the two together.
                // Before it was here, a quarter of the frame was not observed at
                // all and a reader hunting for it went through `style` (audit
                // 3-D). Zero means no host ever reported one, not a free
                // present, and `presents` says over how many of the builds it
                // was averaged: `presents < builds` is frames built that
                // nobody showed, which is the most expensive thing this loop
                // can do and used to be invisible because the previous turn's
                // present was silently reported again in their place.
                // `styled` is the question `style {:.2}ms` cannot answer: a
                // phase that costs milliseconds because it resolved every node
                // and one that costs them because each node is expensive need
                // opposite fixes. `stale` is the same question for the loop
                // itself: components still dirty when the frame ended, i.e. how
                // hard this frame pushed for another one.
                "[phase-stats] {n:.0} builds | reconcile+build {:.2}ms | style {:.2}ms | layout {:.2}ms | compose {:.2}ms | other {other:.2}ms | build {:.2}ms | present {present_ms:.2}ms over {} presents | turn {:.2}ms | styled {:.1}/{:.1} nodes | stale {stale_per_frame:.1}/frame in {} frames",
                ms(t.reconcile),
                ms(t.style),
                ms(t.layout),
                ms(t.compose),
                ms(t.build),
                t.presents,
                ms(t.build) + present_ms,
                resolved_per_frame,
                nodes_per_frame,
                t.stale_frames,
            );
            *t = PhaseTotals::ZERO;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_is_no_longer_zero_by_construction() {
        // The defect this file was changed for: the four marks used to tile the
        // whole measured interval, so `build - Σ phases` was zero however much
        // of the frame lay outside them. Now `unattributed()` closes a stretch
        // that is billed to nothing, and it has to survive into `other`.
        //
        // Asserted on the arithmetic rather than on stdout, because the print
        // is gated on an environment variable a parallel test must not mutate.
        let total = std::time::Duration::from_millis(10);
        let (reconcile, style, layout, compose) = (
            std::time::Duration::from_millis(2),
            std::time::Duration::from_millis(1),
            std::time::Duration::from_millis(1),
            std::time::Duration::from_millis(3),
        );
        let other = total
            .saturating_sub(reconcile)
            .saturating_sub(style)
            .saturating_sub(layout)
            .saturating_sub(compose);
        assert_eq!(other, std::time::Duration::from_millis(3));
    }

    #[test]
    fn phases_can_never_drive_other_below_zero() {
        let total = std::time::Duration::from_millis(1);
        let other = total.saturating_sub(std::time::Duration::from_millis(5));
        assert_eq!(other, std::time::Duration::ZERO);
    }

    #[test]
    fn the_present_the_host_reported_is_what_the_next_frame_reads() {
        // The one thing the shift-by-one has to guarantee: what went in comes
        // back out. The frame that reads it is the one after the turn that
        // wrote it, which is why the report calls this the loop's number.
        record_present_time(std::time::Duration::from_micros(1234));
        assert_eq!(take_present_time(), std::time::Duration::from_micros(1234));
    }

    #[test]
    fn a_present_is_reported_for_one_frame_and_not_for_the_next_one_too() {
        // The defect: `LAST_PRESENT` was peeked, never cleared, so a frame that
        // was built without a present behind it carried its predecessor's
        // number — and `turn = build + present` for that frame added a cost
        // that had already been billed to another frame. A frame with nothing
        // waiting must read zero.
        record_present_time(std::time::Duration::from_micros(7500));
        assert_eq!(
            take_present_time(),
            std::time::Duration::from_micros(7500),
            "the frame after the present reads it"
        );
        assert_eq!(
            take_present_time(),
            std::time::Duration::ZERO,
            "and the frame after that reads nothing, because nothing was shown"
        );
    }

    #[test]
    fn a_present_average_is_taken_over_presents_and_not_over_builds() {
        // The arithmetic `finish` does, asserted where a test can reach it: the
        // print itself is gated on an environment variable a parallel test must
        // not mutate. Three builds, one present of 6 ms — the answer is 6, the
        // cost of a present, not 2, which is the cost of a present smeared over
        // frames that had none.
        let mut totals = PhaseTotals::ZERO;
        totals.builds = 3;
        totals.present = std::time::Duration::from_millis(6);
        totals.presents = 1;
        let present_ms = totals.present.as_secs_f64() * 1000.0 / f64::from(totals.presents);
        assert!((present_ms - 6.0).abs() < 1e-9);
    }

    #[test]
    fn a_clock_nobody_asked_for_counts_nothing() {
        // Every mark is behind `enabled`, so an un-instrumented process pays
        // one `Instant::now()` for the whole frame and no `lap()` at all.
        let clock = FramePhaseClock {
            enabled: false,
            each: false,
            began: std::time::Instant::now(),
            last: std::cell::Cell::new(std::time::Instant::now()),
            reconcile: std::cell::Cell::new(std::time::Duration::ZERO),
            style: std::cell::Cell::new(std::time::Duration::ZERO),
            layout: std::cell::Cell::new(std::time::Duration::ZERO),
            compose: std::cell::Cell::new(std::time::Duration::ZERO),
            styled: std::cell::Cell::new((0, 0)),
            stale: std::cell::Cell::new(0),
        };
        let before = clock.last.get();
        clock.unattributed();
        clock.mark_reconcile();
        clock.mark_images();
        clock.mark_compose();
        clock.note_stale(|| panic!("an instrument nobody switched on must not count"));
        assert_eq!(clock.last.get(), before, "no clock was read");
        assert_eq!(clock.reconcile.get(), std::time::Duration::ZERO);
        assert_eq!(clock.compose.get(), std::time::Duration::ZERO);
        assert_eq!(clock.stale.get(), 0);
    }

    #[test]
    fn compose_accumulates_both_of_its_halves() {
        let clock = FramePhaseClock {
            enabled: true,
            each: false,
            began: std::time::Instant::now(),
            last: std::cell::Cell::new(std::time::Instant::now()),
            reconcile: std::cell::Cell::new(std::time::Duration::ZERO),
            style: std::cell::Cell::new(std::time::Duration::ZERO),
            layout: std::cell::Cell::new(std::time::Duration::ZERO),
            compose: std::cell::Cell::new(std::time::Duration::ZERO),
            styled: std::cell::Cell::new((0, 0)),
            stale: std::cell::Cell::new(0),
        };
        clock.mark_images();
        let after_images = clock.compose.get();
        clock.mark_compose();
        assert!(
            clock.compose.get() >= after_images,
            "the display-list build is added to the image resolution, not over it"
        );
    }
}

//! When a frame actually reached the screen.

/// When a frame actually reached the screen, as the compositor reports it.
///
/// Everything else a runner can measure ends at "the work was handed to the
/// GPU". This is the other side of that handover, and it is the only number in
/// the whole chain that a user could be said to have seen.
///
/// # The clock is the compositor's, and that is the point
///
/// `wp_presentation` states its timestamps in the clock it advertises, which on
/// every Linux compositor is `CLOCK_MONOTONIC` — the same clock input events
/// are stamped with. So a presentation time and a pointer event's timestamp can
/// be subtracted, and the result is the whole of the latency a finger sees:
/// touch to photons, with nothing in the middle assumed. Neither number is this
/// process's own clock, and neither has to be converted to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Presented {
    /// When it was shown, in microseconds on the compositor's clock.
    pub at_micros: u64,
    /// How long the output says a refresh takes, in nanoseconds.
    ///
    /// Zero when the compositor will not say — a variable-refresh output, or
    /// one that does not know. Worth carrying because a frame late by a whole
    /// refresh and a frame late by a millisecond are different faults, and the
    /// only way to tell is to know how long a refresh is on the machine the
    /// measurement was taken on.
    pub refresh_nanos: u32,
    /// Which feedback this answers, counted by the surface that asked.
    ///
    /// A feedback is bound to the next commit, not to a frame number, so the
    /// association between "the frame the runner built" and "the frame the
    /// compositor showed" is by order and by nothing else. Numbering them makes
    /// a dropped or discarded one visible as a gap rather than as a silent
    /// shift of every later reading by one.
    pub seq: u64,
    /// The output's own frame counter at the moment this frame was shown.
    ///
    /// This is the protocol's `seq_hi`/`seq_lo` — MSC, the media stream counter
    /// — and it is the compositor counting **the screen's** refreshes, not this
    /// process counting its own requests. The two answer different questions and
    /// were indistinguishable while this field did not exist:
    ///
    /// * step of one between consecutive presentations — the screen turned over
    ///   as often as we handed it something. Whatever is slow is below us.
    /// * step of N — the screen offered N chances and we took one. That is our
    ///   side, and no amount of blaming the panel will move it.
    ///
    /// Zero when the compositor does not count, which some do not; a run whose
    /// MSC never moves is telling you the field is unsupported, not that the
    /// screen is frozen.
    pub msc: u64,
    /// What the compositor says about *how* it was shown.
    ///
    /// The `wp_presentation_feedback` kind bits: vsync, hardware clock, hardware
    /// completion, zero-copy. Carried rather than dropped because "shown, but
    /// through a copy, off the vsync clock" and "shown by page flip" are
    /// different machines, and a frame budget derived from the wrong one is a
    /// budget for a machine nobody is using.
    pub flags: u32,
}

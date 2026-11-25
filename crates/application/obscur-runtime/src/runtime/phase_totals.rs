#[doc = include_str!("../../docs/api/runtime/frame_phase_clock.doc/PhaseTotals.md")]
#[derive(Default, Clone, Copy)]
pub(super) struct PhaseTotals {
    pub(super) builds: u32,
    pub(super) reconcile: std::time::Duration,
    pub(super) style: std::time::Duration,
    pub(super) layout: std::time::Duration,
    pub(super) compose: std::time::Duration,
    pub(super) build: std::time::Duration,
    pub(super) present: std::time::Duration,
    /// How many of the `builds` had a present waiting for them. See the note
    /// on `LAST_PRESENT`: a present is reported for one frame and one only, so
    /// this is no longer the same as `builds`.
    pub(super) presents: u32,
    pub(super) resolved: u64,
    pub(super) nodes: u64,
    pub(super) stale_frames: u32,
    pub(super) stale: u64,
}

impl PhaseTotals {
    pub(super) const ZERO: Self = Self {
        builds: 0,
        reconcile: std::time::Duration::ZERO,
        style: std::time::Duration::ZERO,
        layout: std::time::Duration::ZERO,
        compose: std::time::Duration::ZERO,
        build: std::time::Duration::ZERO,
        present: std::time::Duration::ZERO,
        presents: 0,
        resolved: 0,
        nodes: 0,
        stale_frames: 0,
        stale: 0,
    };
}

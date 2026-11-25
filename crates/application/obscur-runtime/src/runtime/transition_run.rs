use obscur_motion::{track::Easing, value::MotionValue};

/// The css-transitions-1 §3.1 reversing bookkeeping for one running CSS
/// transition.
#[derive(Debug, Clone, Copy)]
pub(super) struct TransitionRun {
    /// The reversing-adjusted start value: the "logical start state" a reversal
    /// is measured against. For a fresh or non-reversing transition it is the
    /// start value; for a reversal it is the *end* value of the transition it
    /// interrupted (spec §3.1), so repeated reversals compose correctly.
    pub(super) reversing_adjusted_start: MotionValue,
    /// The reversing shortening factor, in `[0, 1]`. `1` for a fresh transition;
    /// less than `1` once a reversal has shortened it.
    pub(super) reversing_factor: f32,
    /// The timing function the transition runs under (its `transition-timing-
    /// function`), needed to read the old transition's output at an interruption.
    pub(super) easing: Easing,
    /// The value the transition heads toward (its end value).
    pub(super) end_value: MotionValue,
}

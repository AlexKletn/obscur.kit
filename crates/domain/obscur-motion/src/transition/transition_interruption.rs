//! The [`TransitionInterruption`] redirect policy for a transition (spec §4.17
//! "interruption").

/// What happens when a new transition targets an element that is *already*
/// transitioning (spec §4.17 "interruption").
///
/// The motion engine must decide how an incoming transition reconciles with the
/// one currently in flight on the same element. This enum names the four
/// policies; the runtime reads it to either replace, turn around, blend into, or
/// suppress the incoming transition. Choosing to continue from the element's
/// present mid-animation value (rather than snapping) is what makes rapid,
/// repeated interactions look seamless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionInterruption {
    /// Discard the current transition and start the new one from its beginning
    /// (spec §4.17). The element may visibly jump to the new start value.
    Restart,
    /// Turn the current transition around so it plays back toward its origin
    /// (spec §4.17). Continues from the present value — used to *undo* an
    /// in-flight enter/exit.
    Reverse,
    /// Start the new transition *from* the element's current mid-animation value
    /// (spec §4.17), producing a seamless redirect with no jump. The smoothest
    /// option and the default.
    Blend,
    /// Let the current transition finish and drop the new one (spec §4.17).
    Ignore,
}

impl TransitionInterruption {
    /// Returns `true` if this policy continues from the element's current value
    /// rather than snapping to a fresh start (spec §4.17 "interruption").
    ///
    /// True for [`Blend`](Self::Blend) and [`Reverse`](Self::Reverse), which
    /// both carry the present mid-animation state forward; false for
    /// [`Restart`](Self::Restart) and [`Ignore`](Self::Ignore).
    #[must_use]
    pub const fn preserves_current_value(&self) -> bool {
        matches!(self, Self::Blend | Self::Reverse)
    }
}

impl Default for TransitionInterruption {
    /// [`Blend`](Self::Blend) — the smoothest UI default: an incoming transition
    /// redirects seamlessly from the element's current value (spec §4.17).
    fn default() -> Self {
        Self::Blend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_blend() {
        assert_eq!(TransitionInterruption::default(), TransitionInterruption::Blend);
    }

    #[test]
    fn preserves_current_value_only_for_blend_and_reverse() {
        assert!(TransitionInterruption::Blend.preserves_current_value());
        assert!(TransitionInterruption::Reverse.preserves_current_value());
        assert!(!TransitionInterruption::Restart.preserves_current_value());
        assert!(!TransitionInterruption::Ignore.preserves_current_value());
    }
}

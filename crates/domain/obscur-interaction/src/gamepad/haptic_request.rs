//! The [`HapticRequest`] describing haptic feedback and its [`HapticPattern`]
//! shape (spec §6.16 haptics).

/// The shape of a haptic feedback effect (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HapticPattern {
    /// A single sharp click.
    Click,
    /// Two clicks in quick succession.
    DoubleClick,
    /// A light, brief tick.
    Tick,
    /// A sustained buzz for the request's duration.
    Buzz,
    /// A custom, platform-defined effect.
    Custom,
}

/// A request for haptic feedback (spec §6.16 haptics).
///
/// A request pairs a [`pattern`](Self::pattern) with an
/// [`intensity`](Self::intensity) in `0.0..=1.0` and a
/// [`duration`](Self::duration_millis) in milliseconds. It is a pure
/// description: this crate emits the request but never actuates hardware or
/// reads a clock. A request is [`silent`](Self::is_silent) when its intensity
/// or duration is zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HapticRequest {
    /// The feedback intensity in `0.0..=1.0`.
    pub intensity: f32,
    /// The effect duration in milliseconds.
    pub duration_millis: u32,
    /// The shape of the effect.
    pub pattern: HapticPattern,
}

impl HapticRequest {
    /// Creates a request for `pattern` at full intensity for 20 milliseconds.
    #[must_use]
    pub const fn new(pattern: HapticPattern) -> Self {
        Self {
            intensity: 1.0,
            duration_millis: 20,
            pattern,
        }
    }

    /// Returns this request with its intensity set, clamped to `0.0..=1.0`
    /// (fluent).
    #[must_use]
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Returns this request with its duration set, in milliseconds (fluent).
    #[must_use]
    pub const fn with_duration(mut self, duration_millis: u32) -> Self {
        self.duration_millis = duration_millis;
        self
    }

    /// Returns the feedback intensity in `0.0..=1.0`.
    #[must_use]
    pub const fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Returns the effect duration in milliseconds.
    #[must_use]
    pub const fn duration_millis(&self) -> u32 {
        self.duration_millis
    }

    /// Returns the shape of the effect.
    #[must_use]
    pub const fn pattern(&self) -> HapticPattern {
        self.pattern
    }

    /// Returns whether this request would produce no perceptible feedback
    /// (intensity `<= 0.0` or zero duration).
    #[must_use]
    pub fn is_silent(&self) -> bool {
        self.intensity <= 0.0 || self.duration_millis == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults() {
        let h = HapticRequest::new(HapticPattern::Click);
        assert_eq!(h.pattern(), HapticPattern::Click);
        assert_eq!(h.intensity(), 1.0);
        assert_eq!(h.duration_millis(), 20);
        assert!(!h.is_silent());
    }

    #[test]
    fn builder_sets_fields() {
        let h = HapticRequest::new(HapticPattern::Buzz)
            .with_intensity(0.5)
            .with_duration(100);
        assert_eq!(h.intensity(), 0.5);
        assert_eq!(h.duration_millis(), 100);
    }

    #[test]
    fn intensity_is_clamped() {
        assert_eq!(
            HapticRequest::new(HapticPattern::Tick)
                .with_intensity(2.0)
                .intensity(),
            1.0
        );
        assert_eq!(
            HapticRequest::new(HapticPattern::Tick)
                .with_intensity(-1.0)
                .intensity(),
            0.0
        );
    }

    #[test]
    fn is_silent_for_zero_intensity_or_duration() {
        assert!(HapticRequest::new(HapticPattern::Click)
            .with_intensity(0.0)
            .is_silent());
        assert!(HapticRequest::new(HapticPattern::Click)
            .with_duration(0)
            .is_silent());
        assert!(!HapticRequest::new(HapticPattern::Click).is_silent());
    }
}

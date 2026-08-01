#![doc = include_str!("haptic_request.doc/_module.md")]

#[doc = include_str!("haptic_request.doc/HapticPattern.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HapticPattern {
    #[doc = include_str!("haptic_request.doc/Click.md")]
    Click,
    #[doc = include_str!("haptic_request.doc/DoubleClick.md")]
    DoubleClick,
    #[doc = include_str!("haptic_request.doc/Tick.md")]
    Tick,
    #[doc = include_str!("haptic_request.doc/Buzz.md")]
    Buzz,
    #[doc = include_str!("haptic_request.doc/Custom.md")]
    Custom,
}

#[doc = include_str!("haptic_request.doc/HapticRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HapticRequest {
    #[doc = include_str!("haptic_request.doc/intensity.md")]
    pub intensity: f32,
    #[doc = include_str!("haptic_request.doc/duration_millis.md")]
    pub duration_millis: u32,
    #[doc = include_str!("haptic_request.doc/pattern.md")]
    pub pattern: HapticPattern,
}

impl HapticRequest {
    #[doc = include_str!("haptic_request.doc/new.md")]
    #[must_use]
    pub const fn new(pattern: HapticPattern) -> Self {
        Self {
            intensity: 1.0,
            duration_millis: 20,
            pattern,
        }
    }

    #[doc = include_str!("haptic_request.doc/with_intensity.md")]
    #[must_use]
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    #[doc = include_str!("haptic_request.doc/with_duration.md")]
    #[must_use]
    pub const fn with_duration(mut self, duration_millis: u32) -> Self {
        self.duration_millis = duration_millis;
        self
    }

    #[doc = include_str!("haptic_request.doc/intensity_2.md")]
    #[must_use]
    pub const fn intensity(&self) -> f32 {
        self.intensity
    }

    #[doc = include_str!("haptic_request.doc/duration_millis_2.md")]
    #[must_use]
    pub const fn duration_millis(&self) -> u32 {
        self.duration_millis
    }

    #[doc = include_str!("haptic_request.doc/pattern_2.md")]
    #[must_use]
    pub const fn pattern(&self) -> HapticPattern {
        self.pattern
    }

    #[doc = include_str!("haptic_request.doc/is_silent.md")]
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
        assert!(
            HapticRequest::new(HapticPattern::Click)
                .with_intensity(0.0)
                .is_silent()
        );
        assert!(
            HapticRequest::new(HapticPattern::Click)
                .with_duration(0)
                .is_silent()
        );
        assert!(!HapticRequest::new(HapticPattern::Click).is_silent());
    }
}

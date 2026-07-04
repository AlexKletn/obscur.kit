//! The live environment `@media`/`@container` conditions are evaluated against
//! at match time: [`MediaEnvironment`].

/// The runtime environment a [`RuleCondition`](crate::matching::RuleCondition)
/// is evaluated against: the viewport and, optionally, the querying container's
/// size, all in logical pixels.
///
/// Any axis left unset (`None`) is *unknown*: a feature query on that axis is
/// treated as non-restrictive (it holds). An [`unbounded`](Self::unbounded)
/// environment therefore leaves every conditional rule applying, which
/// reproduces the pre-gating behaviour for callers that supply no environment.
/// The runtime supplies a concrete [`viewport`](Self::viewport) so that
/// `@media (max-width: 1px)` and friends actually gate.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MediaEnvironment {
    viewport_width: Option<f32>,
    viewport_height: Option<f32>,
    container_width: Option<f32>,
    container_height: Option<f32>,
}

impl MediaEnvironment {
    /// The environment with every axis unknown; every condition holds.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            viewport_width: None,
            viewport_height: None,
            container_width: None,
            container_height: None,
        }
    }

    /// An environment with a known viewport `width`/`height` (logical px) and no
    /// container size.
    #[must_use]
    pub const fn viewport(width: f32, height: f32) -> Self {
        Self {
            viewport_width: Some(width),
            viewport_height: Some(height),
            container_width: None,
            container_height: None,
        }
    }

    /// Returns a copy with the querying container's `width`/`height` (logical
    /// px) set, so `@container` size queries can be evaluated.
    #[must_use]
    pub const fn with_container(mut self, width: f32, height: f32) -> Self {
        self.container_width = Some(width);
        self.container_height = Some(height);
        self
    }

    /// The known viewport width in logical px, if any.
    #[must_use]
    pub const fn viewport_width(&self) -> Option<f32> {
        self.viewport_width
    }

    /// The known viewport height in logical px, if any.
    #[must_use]
    pub const fn viewport_height(&self) -> Option<f32> {
        self.viewport_height
    }

    /// The known container width in logical px, if any.
    #[must_use]
    pub const fn container_width(&self) -> Option<f32> {
        self.container_width
    }

    /// The known container height in logical px, if any.
    #[must_use]
    pub const fn container_height(&self) -> Option<f32> {
        self.container_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_has_no_known_axes() {
        let env = MediaEnvironment::unbounded();
        assert_eq!(env.viewport_width(), None);
        assert_eq!(env.viewport_height(), None);
        assert_eq!(env.container_width(), None);
        assert_eq!(env.container_height(), None);
        assert_eq!(env, MediaEnvironment::default());
    }

    #[test]
    fn viewport_sets_width_and_height_only() {
        let env = MediaEnvironment::viewport(1000.0, 800.0);
        assert_eq!(env.viewport_width(), Some(1000.0));
        assert_eq!(env.viewport_height(), Some(800.0));
        assert_eq!(env.container_width(), None);
    }

    #[test]
    fn with_container_sets_container_axes() {
        let env = MediaEnvironment::viewport(1000.0, 800.0).with_container(200.0, 150.0);
        assert_eq!(env.container_width(), Some(200.0));
        assert_eq!(env.container_height(), Some(150.0));
        // The viewport axes are preserved.
        assert_eq!(env.viewport_width(), Some(1000.0));
    }
}

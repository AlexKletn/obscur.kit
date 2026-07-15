//! The live environment `@media`/`@container` conditions are evaluated against
//! at match time: [`MediaEnvironment`].

/// The runtime environment a [`RuleCondition`](crate::matching::RuleCondition)
/// is evaluated against: the viewport and, optionally, the querying container's
/// size, all in logical pixels.
///
/// An axis left unset (`None`) is *unknown*, and the two axis families resolve
/// an unknown differently (see
/// [`FeatureQuery::holds`](crate::matching::FeatureQuery::holds)):
///
/// - **Viewport** axes are non-restrictive when unknown, so an
///   [`unbounded`](Self::unbounded) environment leaves every `@media` rule
///   applying — the opt-out for callers that supply no environment. The runtime
///   supplies a concrete [`viewport`](Self::viewport) so `@media (max-width:
///   1px)` and friends actually gate.
/// - **Container** axes never hold when unknown, so an `@container` rule
///   applies only when a caller has actually supplied a container size via
///   [`with_container`](Self::with_container).
///
/// # No production caller sets a container size
///
/// [`with_container`](Self::with_container) currently has **no production
/// caller**, so in the live runtime every container axis is unknown and every
/// `@container` rule is inert. This is deliberate and fails *closed*: a
/// container's size is a layout output, but style matching runs over the whole
/// tree strictly *before* layout, and there is no `container-type` property, so
/// nothing can establish a query container. Wiring `@container` for real needs a
/// layout-interleaved restyle pass that this pipeline does not have. The CSS
/// compiler records an `UnsupportedCssFeature` for `@container` on
/// `CompiledStylesheet::unsupported` — the same channel `@import`, `:visited`
/// and `float` use. That field has no production reader today: every consumer
/// takes `.matchable` and drops it, so an author learns nothing unless the host
/// reads it. The rule is recorded, not reported.
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

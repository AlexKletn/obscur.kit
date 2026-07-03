//! Configuration for a headless host: [`HeadlessConfig`].

use obscur_geometry::{ScaleFactor, Size};

/// The configuration for a [`HeadlessHost`](super::HeadlessHost) (spec §7.4).
///
/// A headless run has no OS window and no GPU, so the only inputs it needs are
/// the ones that shape the frame pipeline's geometry: the logical `viewport`
/// each frame is laid out under, and the display `scale` (DPI) that maps logical
/// to physical pixels. Theme, locale, and deterministic-resource configuration
/// are defined by later phases and are intentionally absent here.
///
/// The type is a small plain value (`Copy`), so a host can hold it by value and
/// hand copies out freely. [`Default`] is an 800x600 viewport at
/// [`ScaleFactor::IDENTITY`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadlessConfig {
    /// The logical viewport each frame is laid out under.
    viewport: Size,
    /// The display scale factor mapping logical to physical pixels.
    scale: ScaleFactor,
}

impl HeadlessConfig {
    /// The default logical viewport (800x600) used by [`Default`] and
    /// [`with_viewport`](Self::with_viewport).
    const DEFAULT_VIEWPORT: Size = Size::new(800.0, 600.0);

    /// Creates a configuration with an explicit `viewport` and `scale`.
    #[must_use]
    pub const fn new(viewport: Size, scale: ScaleFactor) -> Self {
        Self { viewport, scale }
    }

    /// Creates a configuration for `viewport` at the identity scale factor.
    ///
    /// This is the common starting point for a headless test: pick a viewport
    /// and, if needed, chain [`with_scale`](Self::with_scale).
    #[must_use]
    pub const fn with_viewport(viewport: Size) -> Self {
        Self::new(viewport, ScaleFactor::IDENTITY)
    }

    /// Returns a copy of this configuration with `scale` replaced.
    #[must_use]
    pub const fn with_scale(self, scale: ScaleFactor) -> Self {
        Self {
            viewport: self.viewport,
            scale,
        }
    }

    /// Returns the logical viewport each frame is laid out under.
    #[must_use]
    pub const fn viewport(self) -> Size {
        self.viewport
    }

    /// Returns the display scale factor mapping logical to physical pixels.
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }
}

impl Default for HeadlessConfig {
    /// An 800x600 logical viewport at [`ScaleFactor::IDENTITY`].
    fn default() -> Self {
        Self::new(Self::DEFAULT_VIEWPORT, ScaleFactor::IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_800x600_at_identity() {
        let config = HeadlessConfig::default();
        assert_eq!(config.viewport(), Size::new(800.0, 600.0));
        assert_eq!(config.scale(), ScaleFactor::IDENTITY);
    }

    #[test]
    fn new_stores_viewport_and_scale() {
        let config = HeadlessConfig::new(Size::new(320.0, 240.0), ScaleFactor::new(2.0));
        assert_eq!(config.viewport(), Size::new(320.0, 240.0));
        assert_eq!(config.scale().get(), 2.0);
    }

    #[test]
    fn with_viewport_starts_at_identity_scale() {
        let config = HeadlessConfig::with_viewport(Size::new(100.0, 200.0));
        assert_eq!(config.viewport(), Size::new(100.0, 200.0));
        assert_eq!(config.scale(), ScaleFactor::IDENTITY);
    }

    #[test]
    fn with_scale_replaces_scale_only() {
        let config = HeadlessConfig::with_viewport(Size::new(100.0, 200.0)).with_scale(
            ScaleFactor::new(3.0),
        );
        assert_eq!(config.viewport(), Size::new(100.0, 200.0));
        assert_eq!(config.scale().get(), 3.0);
    }

    #[test]
    fn is_copy_and_eq() {
        let config = HeadlessConfig::default();
        let copy = config;
        assert_eq!(config, copy);
    }
}

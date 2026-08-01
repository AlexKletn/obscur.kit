#![doc = include_str!("headless_config.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};

#[doc = include_str!("headless_config.doc/HeadlessConfig.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadlessConfig {
    #[doc = include_str!("headless_config.doc/viewport.md")]
    viewport: Size,
    #[doc = include_str!("headless_config.doc/scale.md")]
    scale: ScaleFactor,
}

impl HeadlessConfig {
    #[doc = include_str!("headless_config.doc/DEFAULT_VIEWPORT.md")]
    const DEFAULT_VIEWPORT: Size = Size::new(800.0, 600.0);

    #[doc = include_str!("headless_config.doc/new.md")]
    #[must_use]
    pub const fn new(viewport: Size, scale: ScaleFactor) -> Self {
        Self { viewport, scale }
    }

    #[doc = include_str!("headless_config.doc/with_viewport.md")]
    #[must_use]
    pub const fn with_viewport(viewport: Size) -> Self {
        Self::new(viewport, ScaleFactor::IDENTITY)
    }

    #[doc = include_str!("headless_config.doc/with_scale.md")]
    #[must_use]
    pub const fn with_scale(self, scale: ScaleFactor) -> Self {
        Self {
            viewport: self.viewport,
            scale,
        }
    }

    #[doc = include_str!("headless_config.doc/viewport_2.md")]
    #[must_use]
    pub const fn viewport(self) -> Size {
        self.viewport
    }

    #[doc = include_str!("headless_config.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }
}

impl Default for HeadlessConfig {
    #[doc = include_str!("headless_config.doc/default.md")]
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
        let config = HeadlessConfig::with_viewport(Size::new(100.0, 200.0))
            .with_scale(ScaleFactor::new(3.0));
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

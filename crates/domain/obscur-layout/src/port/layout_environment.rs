#![doc = include_str!("layout_environment.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};

#[doc = include_str!("layout_environment.doc/LayoutEnvironment.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutEnvironment {
    scale_factor: ScaleFactor,
    viewport: Size,
}

impl LayoutEnvironment {
    #[doc = include_str!("layout_environment.doc/new.md")]
    #[must_use]
    pub const fn new(scale_factor: ScaleFactor, viewport: Size) -> Self {
        Self {
            scale_factor,
            viewport,
        }
    }

    #[doc = include_str!("layout_environment.doc/identity.md")]
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            scale_factor: ScaleFactor::IDENTITY,
            viewport: Size::ZERO,
        }
    }

    #[doc = include_str!("layout_environment.doc/scale_factor.md")]
    #[must_use]
    pub const fn scale_factor(self) -> ScaleFactor {
        self.scale_factor
    }

    #[doc = include_str!("layout_environment.doc/viewport.md")]
    #[must_use]
    pub const fn viewport(self) -> Size {
        self.viewport
    }
}

impl Default for LayoutEnvironment {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutEnvironment;
    use obscur_geometry::{ScaleFactor, Size};

    #[test]
    fn new_stores_fields() {
        let env = LayoutEnvironment::new(ScaleFactor::new(2.0), Size::new(800.0, 600.0));
        assert_eq!(env.scale_factor(), ScaleFactor::new(2.0));
        assert_eq!(env.viewport(), Size::new(800.0, 600.0));
    }

    #[test]
    fn identity_uses_neutral_values() {
        let env = LayoutEnvironment::identity();
        assert_eq!(env.scale_factor(), ScaleFactor::IDENTITY);
        assert_eq!(env.viewport(), Size::ZERO);
    }

    #[test]
    fn default_matches_identity() {
        assert_eq!(LayoutEnvironment::default(), LayoutEnvironment::identity());
    }
}

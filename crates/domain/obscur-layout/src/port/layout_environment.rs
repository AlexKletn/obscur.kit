//! `LayoutEnvironment`: the ambient context a measurement happens in.

use obscur_geometry::{ScaleFactor, Size};

/// The ambient context in which an intrinsic measurement is performed.
///
/// Passed to every [`IntrinsicMeasure`](super::IntrinsicMeasure) call so a
/// provider can size content correctly for the current display and viewport.
///
/// For the Phase-1 headless slice this carries only:
/// * `scale_factor` — the device pixel ratio, for DPI-aware measuring;
/// * `viewport` — the viewport size, used later for viewport-relative resolution
///   (e.g. `vw`/`vh` units).
///
/// This type is intentionally minimal and **will grow** in later phases (font
/// context, locale/BiDi, writing mode). It is a plain value: `Copy` and cheap
/// to pass by reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutEnvironment {
    scale_factor: ScaleFactor,
    viewport: Size,
}

impl LayoutEnvironment {
    /// Creates an environment with an explicit scale factor and viewport size.
    #[must_use]
    pub const fn new(scale_factor: ScaleFactor, viewport: Size) -> Self {
        Self {
            scale_factor,
            viewport,
        }
    }

    /// Creates the identity environment: [`ScaleFactor::IDENTITY`] and a
    /// [`Size::ZERO`] viewport. Useful as a neutral default for tests and for
    /// headless measurement where no display context is present.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            scale_factor: ScaleFactor::IDENTITY,
            viewport: Size::ZERO,
        }
    }

    /// The device pixel ratio to measure at.
    #[must_use]
    pub const fn scale_factor(self) -> ScaleFactor {
        self.scale_factor
    }

    /// The viewport size, for later viewport-relative resolution.
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

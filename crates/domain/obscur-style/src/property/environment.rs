//! The typed environment values a host reports and CSS `env()` reads (spec
//! §6.15 insets and virtual keyboard).

use obscur_geometry::Insets;

use super::EnvVar;

#[doc = include_str!("../../docs/api/property/environment.doc/EnvironmentValues.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnvironmentValues {
    /// The safe-area insets: how far in from each viewport edge content must stay
    /// to clear cutouts, rounded corners and system bars.
    pub safe_area: Insets,
    /// The on-screen keyboard's occluded region, as insets from each viewport
    /// edge. A hidden keyboard is all zeroes.
    pub keyboard: Insets,
}

impl EnvironmentValues {
    /// The environment of a host that reports nothing: no cutouts, no keyboard.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            safe_area: Insets::uniform(0.0),
            keyboard: Insets::uniform(0.0),
        }
    }

    /// Returns these values with `safe_area` replaced.
    #[must_use]
    pub const fn with_safe_area(mut self, insets: Insets) -> Self {
        self.safe_area = insets;
        self
    }

    /// Returns these values with the keyboard's occluded region replaced.
    #[must_use]
    pub const fn with_keyboard(mut self, insets: Insets) -> Self {
        self.keyboard = insets;
        self
    }

    #[doc = include_str!("../../docs/api/property/environment.doc/get.md")]
    #[must_use]
    pub fn get(self, var: EnvVar) -> f32 {
        match var {
            EnvVar::SafeAreaInsetTop => self.safe_area.top,
            EnvVar::SafeAreaInsetRight => self.safe_area.right,
            EnvVar::SafeAreaInsetBottom => self.safe_area.bottom,
            EnvVar::SafeAreaInsetLeft => self.safe_area.left,
            EnvVar::KeyboardInsetTop => self.keyboard.top,
            EnvVar::KeyboardInsetRight => self.keyboard.right,
            EnvVar::KeyboardInsetBottom => self.keyboard.bottom,
            EnvVar::KeyboardInsetLeft => self.keyboard.left,
            EnvVar::KeyboardInsetWidth => self.keyboard.left + self.keyboard.right,
            EnvVar::KeyboardInsetHeight => self.keyboard.top + self.keyboard.bottom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_that_reports_nothing_has_no_insets() {
        // The default is the right answer for a screen with no cutout and no
        // keyboard, not a placeholder for one.
        let env = EnvironmentValues::default();
        for var in [
            EnvVar::SafeAreaInsetTop,
            EnvVar::SafeAreaInsetBottom,
            EnvVar::KeyboardInsetHeight,
        ] {
            assert_eq!(env.get(var), 0.0, "{}", var.name());
        }
        assert_eq!(env, EnvironmentValues::none());
    }

    #[test]
    fn each_edge_is_reported_independently() {
        let env = EnvironmentValues::none().with_safe_area(Insets {
            top: 44.0,
            right: 1.0,
            bottom: 34.0,
            left: 2.0,
        });
        assert_eq!(env.get(EnvVar::SafeAreaInsetTop), 44.0);
        assert_eq!(env.get(EnvVar::SafeAreaInsetRight), 1.0);
        assert_eq!(env.get(EnvVar::SafeAreaInsetBottom), 34.0);
        assert_eq!(env.get(EnvVar::SafeAreaInsetLeft), 2.0);
    }

    #[test]
    fn the_keyboard_is_separate_from_the_safe_area() {
        // A keyboard covering the bottom 300px must not read as a safe-area
        // inset: content clears a home indicator *and* the keyboard, and the two
        // are different distances.
        let env = EnvironmentValues::none()
            .with_safe_area(Insets::uniform(10.0))
            .with_keyboard(Insets {
                top: 0.0,
                right: 0.0,
                bottom: 300.0,
                left: 0.0,
            });
        assert_eq!(env.get(EnvVar::SafeAreaInsetBottom), 10.0);
        assert_eq!(env.get(EnvVar::KeyboardInsetBottom), 300.0);
        assert_eq!(env.get(EnvVar::KeyboardInsetHeight), 300.0);
    }
}

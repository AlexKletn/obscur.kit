//! The typed environment values a host reports and CSS `env()` reads (spec
//! §6.15 insets and virtual keyboard).

use obscur_geometry::Insets;

/// One CSS environment value, named by `env(<name>)` (spec §6.15; css-env-1).
///
/// These are **reported by the host, not chosen by the application**: the notch
/// a phone's screen is cut by and the on-screen keyboard's geometry are facts
/// about the device and the moment, and an app can only lay itself out around
/// them. That is why they arrive as a typed environment rather than as
/// properties an author sets.
///
/// A name this enum does not list is not an environment value here, and a
/// declaration using one is dropped as invalid rather than silently resolving to
/// zero — an unknown name is a typo or a value the kit does not model, and both
/// are better as a dropped declaration than as an inset that quietly is not
/// there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvVar {
    /// `safe-area-inset-top` — the distance from the top edge of the viewport to
    /// the first row that is not obscured by a cutout, a rounded corner or a
    /// system bar.
    SafeAreaInsetTop,
    /// `safe-area-inset-right`.
    SafeAreaInsetRight,
    /// `safe-area-inset-bottom` — a home indicator or gesture bar lives here.
    SafeAreaInsetBottom,
    /// `safe-area-inset-left`.
    SafeAreaInsetLeft,
    /// `keyboard-inset-top` — the top edge of the on-screen keyboard, measured
    /// from the top of the viewport, or the viewport height when no keyboard is
    /// shown.
    KeyboardInsetTop,
    /// `keyboard-inset-right`.
    KeyboardInsetRight,
    /// `keyboard-inset-bottom`.
    KeyboardInsetBottom,
    /// `keyboard-inset-left`.
    KeyboardInsetLeft,
    /// `keyboard-inset-width` — the on-screen keyboard's width, `0` when hidden.
    KeyboardInsetWidth,
    /// `keyboard-inset-height` — the on-screen keyboard's height, `0` when
    /// hidden. This is the one an app usually wants: it is how much of the
    /// viewport the keyboard has taken.
    KeyboardInsetHeight,
}

impl EnvVar {
    /// Resolves the CSS name inside `env(…)`, or [`None`] for a name this kit
    /// does not model.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "safe-area-inset-top" => Some(Self::SafeAreaInsetTop),
            "safe-area-inset-right" => Some(Self::SafeAreaInsetRight),
            "safe-area-inset-bottom" => Some(Self::SafeAreaInsetBottom),
            "safe-area-inset-left" => Some(Self::SafeAreaInsetLeft),
            "keyboard-inset-top" => Some(Self::KeyboardInsetTop),
            "keyboard-inset-right" => Some(Self::KeyboardInsetRight),
            "keyboard-inset-bottom" => Some(Self::KeyboardInsetBottom),
            "keyboard-inset-left" => Some(Self::KeyboardInsetLeft),
            "keyboard-inset-width" => Some(Self::KeyboardInsetWidth),
            "keyboard-inset-height" => Some(Self::KeyboardInsetHeight),
            _ => None,
        }
    }

    /// The CSS name this value is written as.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SafeAreaInsetTop => "safe-area-inset-top",
            Self::SafeAreaInsetRight => "safe-area-inset-right",
            Self::SafeAreaInsetBottom => "safe-area-inset-bottom",
            Self::SafeAreaInsetLeft => "safe-area-inset-left",
            Self::KeyboardInsetTop => "keyboard-inset-top",
            Self::KeyboardInsetRight => "keyboard-inset-right",
            Self::KeyboardInsetBottom => "keyboard-inset-bottom",
            Self::KeyboardInsetLeft => "keyboard-inset-left",
            Self::KeyboardInsetWidth => "keyboard-inset-width",
            Self::KeyboardInsetHeight => "keyboard-inset-height",
        }
    }
}

/// The environment values in force, in logical pixels (spec §6.15).
///
/// All zero by default, and that default is **correct rather than merely safe**:
/// a display with no cutout and no on-screen keyboard genuinely has no insets, so
/// a desktop host that reports nothing gets the right answer without knowing this
/// type exists. A host that does know — a tablet shell, a mobile platform — sets
/// what it measures, and the values change over the app's life as the keyboard
/// comes and goes or the device is rotated.
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

    /// The logical-pixel value of `var`.
    ///
    /// `keyboard-inset-width` / `-height` are derived from the occluded region
    /// rather than stored: they are the extent of the keyboard, which is what the
    /// left/right and top/bottom insets already say.
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

    #[test]
    fn names_round_trip() {
        for var in [
            EnvVar::SafeAreaInsetTop,
            EnvVar::SafeAreaInsetRight,
            EnvVar::SafeAreaInsetBottom,
            EnvVar::SafeAreaInsetLeft,
            EnvVar::KeyboardInsetTop,
            EnvVar::KeyboardInsetRight,
            EnvVar::KeyboardInsetBottom,
            EnvVar::KeyboardInsetLeft,
            EnvVar::KeyboardInsetWidth,
            EnvVar::KeyboardInsetHeight,
        ] {
            assert_eq!(EnvVar::from_name(var.name()), Some(var), "{}", var.name());
        }
    }

    #[test]
    fn an_unmodelled_name_is_not_an_environment_value() {
        // Dropped as invalid, not resolved to zero: a typo must not look like an
        // inset that happens to be absent.
        assert_eq!(EnvVar::from_name("safe-area-inset-topp"), None);
        assert_eq!(EnvVar::from_name("titlebar-area-height"), None);
        assert_eq!(EnvVar::from_name(""), None);
    }
}

#![doc = include_str!("environment.doc/_module.md")]

use obscur_geometry::Insets;

#[doc = include_str!("environment.doc/EnvVar.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvVar {
    #[doc = include_str!("environment.doc/SafeAreaInsetTop.md")]
    SafeAreaInsetTop,
    #[doc = include_str!("environment.doc/SafeAreaInsetRight.md")]
    SafeAreaInsetRight,
    #[doc = include_str!("environment.doc/SafeAreaInsetBottom.md")]
    SafeAreaInsetBottom,
    #[doc = include_str!("environment.doc/SafeAreaInsetLeft.md")]
    SafeAreaInsetLeft,
    #[doc = include_str!("environment.doc/KeyboardInsetTop.md")]
    KeyboardInsetTop,
    #[doc = include_str!("environment.doc/KeyboardInsetRight.md")]
    KeyboardInsetRight,
    #[doc = include_str!("environment.doc/KeyboardInsetBottom.md")]
    KeyboardInsetBottom,
    #[doc = include_str!("environment.doc/KeyboardInsetLeft.md")]
    KeyboardInsetLeft,
    #[doc = include_str!("environment.doc/KeyboardInsetWidth.md")]
    KeyboardInsetWidth,
    #[doc = include_str!("environment.doc/KeyboardInsetHeight.md")]
    KeyboardInsetHeight,
}

impl EnvVar {
    #[doc = include_str!("environment.doc/from_name.md")]
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

    #[doc = include_str!("environment.doc/name.md")]
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

#[doc = include_str!("environment.doc/EnvironmentValues.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnvironmentValues {
    #[doc = include_str!("environment.doc/safe_area.md")]
    pub safe_area: Insets,
    #[doc = include_str!("environment.doc/keyboard.md")]
    pub keyboard: Insets,
}

impl EnvironmentValues {
    #[doc = include_str!("environment.doc/none.md")]
    #[must_use]
    pub const fn none() -> Self {
        Self {
            safe_area: Insets::uniform(0.0),
            keyboard: Insets::uniform(0.0),
        }
    }

    #[doc = include_str!("environment.doc/with_safe_area.md")]
    #[must_use]
    pub const fn with_safe_area(mut self, insets: Insets) -> Self {
        self.safe_area = insets;
        self
    }

    #[doc = include_str!("environment.doc/with_keyboard.md")]
    #[must_use]
    pub const fn with_keyboard(mut self, insets: Insets) -> Self {
        self.keyboard = insets;
        self
    }

    #[doc = include_str!("environment.doc/get.md")]
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

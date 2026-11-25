#[doc = include_str!("../../docs/api/property/environment.doc/EnvVar.md")]
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

#[cfg(test)]
mod tests {
    use super::EnvVar;

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

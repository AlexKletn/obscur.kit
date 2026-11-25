//! Everything the desktop said about appearance, in one value.

use obscur_scene::color::Color;

use super::{ColorScheme, Contrast, SystemFont, SystemPalette, WindowDecoration};

#[doc = include_str!("../../docs/api/appearance/system_appearance.doc/SystemAppearance.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemAppearance {
    /// Light or dark, if the desktop stated a preference.
    pub color_scheme: Option<ColorScheme>,
    /// Whether more contrast was asked for.
    pub contrast: Option<Contrast>,
    #[doc = include_str!("../../docs/api/appearance/system_appearance.doc/accent.md")]
    pub accent: Option<Color>,
    /// The named colours, which on some desktops is none of them.
    pub palette: SystemPalette,
    /// Ordinary window corner radius in logical pixels.
    pub window_radius: Option<f32>,
    /// Desktop title bar preferences for client-side decorations. The selected provider supplies the whole decoration so that icon conventions follow its button layout.
    pub window_decoration: Option<WindowDecoration>,
    /// The interface font.
    pub font: Option<SystemFont>,
    /// The fixed-width font, for code and terminals.
    pub monospace_font: Option<SystemFont>,
    #[doc = include_str!("../../docs/api/appearance/system_appearance.doc/text_scale.md")]
    pub text_scale: Option<f32>,
    /// The pointer cursor's size in pixels.
    pub cursor_size: Option<u32>,
    #[doc = include_str!("../../docs/api/appearance/system_appearance.doc/icon_theme.md")]
    pub icon_theme: Option<Box<str>>,
    #[doc = include_str!("../../docs/api/appearance/system_appearance.doc/reduced_motion.md")]
    pub reduced_motion: Option<bool>,
}

impl SystemAppearance {
    /// Whether the desktop answered nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.color_scheme.is_none()
            && self.contrast.is_none()
            && self.accent.is_none()
            && self.palette.is_empty()
            && self.window_radius.is_none()
            && self.window_decoration.is_none()
            && self.font.is_none()
            && self.monospace_font.is_none()
            && self.text_scale.is_none()
            && self.cursor_size.is_none()
            && self.icon_theme.is_none()
            && self.reduced_motion.is_none()
    }

    #[doc = include_str!("../../docs/api/appearance/system_appearance.doc/merge_under.md")]
    #[must_use]
    pub fn merge_under(mut self, weaker: &Self) -> Self {
        self.color_scheme = self.color_scheme.or(weaker.color_scheme);
        self.contrast = self.contrast.or(weaker.contrast);
        self.accent = self.accent.or(weaker.accent);
        self.window_radius = self.window_radius.or(weaker.window_radius);
        self.window_decoration = self
            .window_decoration
            .or_else(|| weaker.window_decoration.clone());
        self.font = self.font.take().or_else(|| weaker.font.clone());
        self.monospace_font = self
            .monospace_font
            .take()
            .or_else(|| weaker.monospace_font.clone());
        self.text_scale = self.text_scale.or(weaker.text_scale);
        self.cursor_size = self.cursor_size.or(weaker.cursor_size);
        self.icon_theme = self.icon_theme.take().or_else(|| weaker.icon_theme.clone());
        self.reduced_motion = self.reduced_motion.or(weaker.reduced_motion);

        let palette = &mut self.palette;
        palette.window_background = palette
            .window_background
            .or(weaker.palette.window_background);
        palette.window_foreground = palette
            .window_foreground
            .or(weaker.palette.window_foreground);
        palette.view_background = palette.view_background.or(weaker.palette.view_background);
        palette.view_foreground = palette.view_foreground.or(weaker.palette.view_foreground);
        palette.selection_background = palette
            .selection_background
            .or(weaker.palette.selection_background);
        palette.selection_foreground = palette
            .selection_foreground
            .or(weaker.palette.selection_foreground);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dark() -> SystemAppearance {
        SystemAppearance {
            color_scheme: Some(ColorScheme::Dark),
            ..SystemAppearance::default()
        }
    }

    #[test]
    fn an_explicitly_empty_decoration_keeps_its_layout_during_merge() {
        use super::super::{WindowButton, WindowButtonLayout};

        let strong = SystemAppearance {
            window_decoration: Some(WindowDecoration {
                button_layout: WindowButtonLayout {
                    left: Some(vec![]),
                    right: Some(vec![]),
                },
                button_icons: Default::default(),
            }),
            ..SystemAppearance::default()
        };
        let weak = SystemAppearance {
            window_decoration: Some(WindowDecoration {
                button_layout: WindowButtonLayout {
                    right: Some(vec![WindowButton::Close]),
                    ..WindowButtonLayout::default()
                },
                button_icons: Default::default(),
            }),
            ..SystemAppearance::default()
        };

        assert!(!strong.is_empty());
        assert_eq!(strong.clone().merge_under(&weak), strong);
        assert_eq!(SystemAppearance::default().merge_under(&weak), weak);
    }

    #[test]
    fn a_desktop_that_answered_nothing_is_empty() {
        assert!(SystemAppearance::default().is_empty());
        assert!(!dark().is_empty());
    }

    #[test]
    fn merging_fills_gaps_and_never_overwrites() {
        // The composite source's whole behaviour: the stronger provider answered
        // the scheme, the weaker one knows the palette, and neither loses.
        let strong = dark();
        let weak = SystemAppearance {
            color_scheme: Some(ColorScheme::Light),
            accent: Some(Color::rgb(0.0, 0.4, 1.0)),
            palette: SystemPalette {
                window_background: Some(Color::rgb(0.1, 0.1, 0.1)),
                ..SystemPalette::default()
            },
            ..SystemAppearance::default()
        };

        let merged = strong.merge_under(&weak);
        assert_eq!(
            merged.color_scheme,
            Some(ColorScheme::Dark),
            "the stronger answer stands"
        );
        assert_eq!(
            merged.accent,
            Some(Color::rgb(0.0, 0.4, 1.0)),
            "and the gap it left is filled"
        );
        assert_eq!(
            merged.palette.window_background,
            Some(Color::rgb(0.1, 0.1, 0.1))
        );
    }

    #[test]
    fn merging_under_nothing_changes_nothing() {
        let strong = dark();
        assert_eq!(
            strong.clone().merge_under(&SystemAppearance::default()),
            strong
        );
    }

    #[test]
    fn merging_nothing_under_an_answer_takes_the_answer() {
        let weak = dark();
        assert_eq!(SystemAppearance::default().merge_under(&weak), weak);
    }

    #[test]
    fn the_palette_merges_field_by_field_rather_than_whole() {
        // Two sources each knowing half a palette is the realistic case, and a
        // whole-struct `or` would drop the half that arrived second.
        let strong = SystemAppearance {
            palette: SystemPalette {
                window_background: Some(Color::rgb(1.0, 1.0, 1.0)),
                ..SystemPalette::default()
            },
            ..SystemAppearance::default()
        };
        let weak = SystemAppearance {
            palette: SystemPalette {
                window_foreground: Some(Color::rgb(0.0, 0.0, 0.0)),
                ..SystemPalette::default()
            },
            ..SystemAppearance::default()
        };
        assert_eq!(strong.merge_under(&weak).palette.stated(), 2);
    }
}

#![doc = include_str!("system_appearance.doc/_module.md")]

use obscur_scene::color::Color;

use super::{ColorScheme, Contrast, SystemFont, SystemPalette};

#[doc = include_str!("system_appearance.doc/SystemAppearance.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemAppearance {
    #[doc = include_str!("system_appearance.doc/color_scheme.md")]
    pub color_scheme: Option<ColorScheme>,
    #[doc = include_str!("system_appearance.doc/contrast.md")]
    pub contrast: Option<Contrast>,
    #[doc = include_str!("system_appearance.doc/accent.md")]
    pub accent: Option<Color>,
    #[doc = include_str!("system_appearance.doc/palette.md")]
    pub palette: SystemPalette,
    #[doc = include_str!("system_appearance.doc/font.md")]
    pub font: Option<SystemFont>,
    #[doc = include_str!("system_appearance.doc/monospace_font.md")]
    pub monospace_font: Option<SystemFont>,
    #[doc = include_str!("system_appearance.doc/text_scale.md")]
    pub text_scale: Option<f32>,
    #[doc = include_str!("system_appearance.doc/cursor_size.md")]
    pub cursor_size: Option<u32>,
    #[doc = include_str!("system_appearance.doc/icon_theme.md")]
    pub icon_theme: Option<Box<str>>,
    #[doc = include_str!("system_appearance.doc/reduced_motion.md")]
    pub reduced_motion: Option<bool>,
}

impl SystemAppearance {
    #[doc = include_str!("system_appearance.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.color_scheme.is_none()
            && self.contrast.is_none()
            && self.accent.is_none()
            && self.palette.is_empty()
            && self.font.is_none()
            && self.monospace_font.is_none()
            && self.text_scale.is_none()
            && self.cursor_size.is_none()
            && self.icon_theme.is_none()
            && self.reduced_motion.is_none()
    }

    #[doc = include_str!("system_appearance.doc/merge_under.md")]
    #[must_use]
    pub fn merge_under(mut self, weaker: &Self) -> Self {
        self.color_scheme = self.color_scheme.or(weaker.color_scheme);
        self.contrast = self.contrast.or(weaker.contrast);
        self.accent = self.accent.or(weaker.accent);
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

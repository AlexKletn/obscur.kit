#![doc = include_str!("theme_vars.doc/_module.md")]

use obscur_scene::color::Color;
use obscur_system::appearance::{SystemAppearance, SystemFont};

#[doc = include_str!("theme_vars.doc/PREFIX.md")]
pub const PREFIX: &str = "--obscur-sys-";

#[doc = include_str!("theme_vars.doc/ThemeVars.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThemeVars {
    entries: Vec<(String, String)>,
}

impl ThemeVars {
    #[doc = include_str!("theme_vars.doc/from_appearance.md")]
    #[must_use]
    pub fn from_appearance(appearance: &SystemAppearance) -> Self {
        let mut vars = Self::default();

        if let Some(scheme) = appearance.color_scheme {
            vars.push("color-scheme", scheme.css_keyword());
        }
        if let Some(contrast) = appearance.contrast {
            vars.push("contrast", contrast.css_keyword());
        }
        if let Some(accent) = appearance.accent {
            vars.push("accent", &css_color(accent));
        }

        let palette = &appearance.palette;
        vars.push_color("window-bg", palette.window_background);
        vars.push_color("window-fg", palette.window_foreground);
        vars.push_color("view-bg", palette.view_background);
        vars.push_color("view-fg", palette.view_foreground);
        vars.push_color("selection-bg", palette.selection_background);
        vars.push_color("selection-fg", palette.selection_foreground);

        vars.push_font("font", appearance.font.as_ref());
        vars.push_font("mono", appearance.monospace_font.as_ref());

        if let Some(scale) = appearance.text_scale {
            vars.push("text-scale", &trim_number(scale));
        }
        if let Some(size) = appearance.cursor_size {
            vars.push("cursor-size", &format!("{size}px"));
        }
        if let Some(theme) = appearance.icon_theme.as_deref() {
            vars.push("icon-theme", theme);
        }
        if let Some(reduced) = appearance.reduced_motion {
            // A number rather than a keyword, so a stylesheet can multiply a
            // duration by it and get zero without a second rule.
            vars.push("reduced-motion", if reduced { "1" } else { "0" });
        }

        vars
    }

    #[doc = include_str!("theme_vars.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    #[doc = include_str!("theme_vars.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("theme_vars.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[doc = include_str!("theme_vars.doc/get.md")]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    #[doc = include_str!("theme_vars.doc/stylesheet.md")]
    #[must_use]
    pub fn stylesheet(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let mut out = String::from(":root {");
        for (name, value) in &self.entries {
            out.push(' ');
            out.push_str(name);
            out.push_str(": ");
            out.push_str(value);
            out.push(';');
        }
        out.push_str(" }\n");
        out
    }

    #[doc = include_str!("theme_vars.doc/push.md")]
    fn push(&mut self, suffix: &str, value: &str) {
        self.entries
            .push((format!("{PREFIX}{suffix}"), value.to_owned()));
    }

    #[doc = include_str!("theme_vars.doc/push_color.md")]
    fn push_color(&mut self, suffix: &str, color: Option<Color>) {
        if let Some(color) = color {
            self.push(suffix, &css_color(color));
        }
    }

    #[doc = include_str!("theme_vars.doc/push_font.md")]
    fn push_font(&mut self, suffix: &str, font: Option<&SystemFont>) {
        let Some(font) = font else {
            return;
        };
        // Quoted, because a family is one name and a bare `Noto Sans Mono`
        // would parse as a list of three.
        self.push(&format!("{suffix}-family"), &format!("\"{}\"", font.family));
        if let Some(px) = font.size_px() {
            self.push(&format!("{suffix}-size"), &format!("{}px", trim_number(px)));
        }
    }
}

#[doc = include_str!("theme_vars.doc/css_color.md")]
fn css_color(color: Color) -> String {
    // Clamped first, so the cast below cannot see a value outside 0..=255 and
    // the truncation clippy warns about cannot happen.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let (r, g, b) = (channel(color.r), channel(color.g), channel(color.b));
    if color.a >= 1.0 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("rgba({r}, {g}, {b}, {})", trim_number(color.a))
    }
}

#[doc = include_str!("theme_vars.doc/trim_number.md")]
fn trim_number(value: f32) -> String {
    // Three decimals is finer than any of these values is measured to, and a
    // trailing `.0` in a stylesheet is noise a reader has to skip.
    let text = format!("{value:.3}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use obscur_system::appearance::{ColorScheme, Contrast, SystemPalette};

    use super::*;

    fn full() -> SystemAppearance {
        SystemAppearance {
            color_scheme: Some(ColorScheme::Dark),
            contrast: Some(Contrast::High),
            accent: Some(Color::rgb(0.208, 0.518, 0.894)),
            palette: SystemPalette {
                window_background: Some(Color::rgb(0.165, 0.18, 0.196)),
                window_foreground: Some(Color::rgb(0.988, 0.988, 0.988)),
                ..SystemPalette::default()
            },
            font: Some(SystemFont::new("Noto Sans", Some(10.0))),
            monospace_font: Some(SystemFont::new("Hack", Some(10.0))),
            text_scale: Some(1.25),
            cursor_size: Some(24),
            icon_theme: Some("breeze-dark".into()),
            reduced_motion: Some(true),
        }
    }

    #[test]
    fn a_desktop_that_said_nothing_produces_no_variables_and_no_sheet() {
        // The contract that makes the rest safe: absence travels all the way to
        // the CSS, so an author's own fallback is what applies.
        let vars = ThemeVars::from_appearance(&SystemAppearance::default());
        assert!(vars.is_empty());
        assert_eq!(vars.stylesheet(), "");
    }

    #[test]
    fn only_what_the_desktop_said_is_emitted() {
        let partial = SystemAppearance {
            color_scheme: Some(ColorScheme::Light),
            ..SystemAppearance::default()
        };
        let vars = ThemeVars::from_appearance(&partial);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars.get("--obscur-sys-color-scheme"), Some("light"));
        assert_eq!(vars.get("--obscur-sys-accent"), None);
    }

    #[test]
    fn every_name_carries_the_prefix() {
        for (name, _) in ThemeVars::from_appearance(&full()).iter() {
            assert!(name.starts_with(PREFIX), "{name} is unprefixed");
        }
    }

    #[test]
    fn a_colour_is_hex_and_a_translucent_one_is_not() {
        assert_eq!(css_color(Color::rgb(0.0, 0.0, 0.0)), "#000000");
        assert_eq!(css_color(Color::rgb(1.0, 1.0, 1.0)), "#ffffff");
        assert_eq!(
            css_color(Color::rgba(1.0, 0.0, 0.0, 0.5)),
            "rgba(255, 0, 0, 0.5)"
        );
    }

    #[test]
    fn a_font_size_arrives_in_pixels_not_points() {
        // 10pt is 13.333px, and a stylesheet has no idea what a point is.
        let vars = ThemeVars::from_appearance(&full());
        assert_eq!(vars.get("--obscur-sys-font-size"), Some("13.333px"));
        assert_eq!(vars.get("--obscur-sys-font-family"), Some("\"Noto Sans\""));
    }

    #[test]
    fn a_family_is_quoted_so_its_spaces_do_not_make_a_list() {
        let vars = ThemeVars::from_appearance(&SystemAppearance {
            font: Some(SystemFont::new("Noto Sans Mono", None)),
            ..SystemAppearance::default()
        });
        assert_eq!(
            vars.get("--obscur-sys-font-family"),
            Some("\"Noto Sans Mono\"")
        );
        assert_eq!(
            vars.get("--obscur-sys-font-size"),
            None,
            "no size was stated, so none is emitted"
        );
    }

    #[test]
    fn the_sheet_is_one_root_rule_that_parses_as_declarations() {
        let sheet = ThemeVars::from_appearance(&full()).stylesheet();
        assert!(sheet.starts_with(":root {"), "{sheet}");
        assert!(sheet.trim_end().ends_with('}'), "{sheet}");
        // One `;` per variable, so nothing was concatenated into its neighbour.
        let vars = ThemeVars::from_appearance(&full());
        assert_eq!(sheet.matches(';').count(), vars.len());
    }

    #[test]
    fn reduced_motion_is_a_number_a_stylesheet_can_multiply_by() {
        let vars = ThemeVars::from_appearance(&full());
        assert_eq!(vars.get("--obscur-sys-reduced-motion"), Some("1"));
        let moving = ThemeVars::from_appearance(&SystemAppearance {
            reduced_motion: Some(false),
            ..SystemAppearance::default()
        });
        assert_eq!(moving.get("--obscur-sys-reduced-motion"), Some("0"));
    }

    #[test]
    fn numbers_do_not_carry_a_trailing_zero() {
        assert_eq!(trim_number(1.0), "1");
        assert_eq!(trim_number(1.25), "1.25");
        assert_eq!(trim_number(0.0), "0");
        assert_eq!(trim_number(13.333_333), "13.333");
    }

    #[test]
    fn the_keywords_are_the_ones_css_media_queries_use() {
        // So `--obscur-sys-contrast` can be compared against the same word an
        // author would write in `@media (prefers-contrast: more)`.
        let vars = ThemeVars::from_appearance(&full());
        assert_eq!(vars.get("--obscur-sys-color-scheme"), Some("dark"));
        assert_eq!(vars.get("--obscur-sys-contrast"), Some("more"));
    }
}

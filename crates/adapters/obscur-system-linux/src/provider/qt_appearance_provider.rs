#![doc = include_str!("qt_appearance_provider.doc/_module.md")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use obscur_capability::SupportLevel;
use obscur_scene::color::Color;
use obscur_system::appearance::{
    AppearanceProvider, ColorScheme, SystemAppearance, SystemFont, SystemPalette,
};

#[doc = include_str!("qt_appearance_provider.doc/QtAppearanceProvider.md")]
#[derive(Debug, Clone)]
pub struct QtAppearanceProvider {
    path: PathBuf,
}

impl QtAppearanceProvider {
    #[doc = include_str!("qt_appearance_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::at(default_path())
    }

    #[doc = include_str!("qt_appearance_provider.doc/at.md")]
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[doc = include_str!("qt_appearance_provider.doc/path.md")]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for QtAppearanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AppearanceProvider for QtAppearanceProvider {
    fn appearance(&self) -> SystemAppearance {
        std::fs::read_to_string(&self.path)
            .map_or_else(|_| SystemAppearance::default(), |text| parse(&text))
    }

    fn support(&self) -> SupportLevel {
        if self.path.is_file() {
            // The only source on this platform that states a whole palette, so
            // when the file is there the answer is the complete shape.
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }

    #[doc = include_str!("qt_appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        // Unavailable, and this is the asymmetry worth stating out loud: the
        // source that knows the most is the one that cannot tell you when it
        // changes. `kdeglobals` is a file and a file emits nothing. Watching it
        // would mean `inotify`, or KDE's own `KConfigWatcher` bus traffic —
        // either is a dependency this adapter does not have, and neither is
        // hidden behind a `Native` that would make a host stop polling.
        SupportLevel::Unavailable
    }
}

#[doc = include_str!("qt_appearance_provider.doc/default_path.md")]
fn default_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("kdeglobals")
}

#[doc = include_str!("qt_appearance_provider.doc/parse.md")]
fn parse(text: &str) -> SystemAppearance {
    let ini = read_ini(text);
    let get = |section: &str, key: &str| {
        ini.get(&(section.to_owned(), key.to_owned()))
            .map(String::as_str)
    };

    let palette = SystemPalette {
        window_background: get("Colors:Window", "BackgroundNormal").and_then(parse_rgb),
        window_foreground: get("Colors:Window", "ForegroundNormal").and_then(parse_rgb),
        view_background: get("Colors:View", "BackgroundNormal").and_then(parse_rgb),
        view_foreground: get("Colors:View", "ForegroundNormal").and_then(parse_rgb),
        selection_background: get("Colors:Selection", "BackgroundNormal").and_then(parse_rgb),
        selection_foreground: get("Colors:Selection", "ForegroundNormal").and_then(parse_rgb),
    };

    SystemAppearance {
        // The palette decides, and only the scheme's *name* is consulted when
        // there is no palette: a theme called "BreezeDark" is evidence, but a
        // measured background is proof, and plenty of dark themes are not
        // called dark.
        color_scheme: palette
            .window_background
            .map(|bg| {
                if luminance(bg) < 0.5 {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                }
            })
            .or_else(|| {
                get("General", "ColorScheme").map(|name| {
                    if name.to_ascii_lowercase().contains("dark") {
                        ColorScheme::Dark
                    } else {
                        ColorScheme::Light
                    }
                })
            }),
        contrast: None,
        accent: get("General", "AccentColor").and_then(parse_rgb),
        palette,
        font: get("General", "font").and_then(parse_font),
        monospace_font: get("General", "fixed").and_then(parse_font),
        text_scale: None,
        cursor_size: None,
        icon_theme: get("Icons", "Theme").map(Into::into),
        // KDE states a duration multiplier; zero means the user turned
        // animation off, which is the same request `prefers-reduced-motion`
        // makes.
        reduced_motion: get("KDE", "AnimationDurationFactor")
            .and_then(|value| value.trim().parse::<f32>().ok())
            .map(|factor| factor <= f32::EPSILON),
    }
}

#[doc = include_str!("qt_appearance_provider.doc/read_ini.md")]
fn read_ini(text: &str) -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            name.clone_into(&mut section);
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            out.insert(
                (section.clone(), key.trim().to_owned()),
                value.trim().to_owned(),
            );
        }
    }
    out
}

#[doc = include_str!("qt_appearance_provider.doc/parse_rgb.md")]
fn parse_rgb(value: &str) -> Option<Color> {
    let mut parts = value.split(',').map(|part| part.trim().parse::<u8>());
    let r = parts.next()?.ok()?;
    let g = parts.next()?.ok()?;
    let b = parts.next()?.ok()?;
    // A fourth component is alpha where KDE writes one; anything beyond that is
    // not this format and the value is refused rather than half-read.
    let a = match parts.next() {
        Some(Ok(a)) => f32::from(a) / 255.0,
        Some(Err(_)) => return None,
        None => 1.0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(Color::rgba(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        a,
    ))
}

#[doc = include_str!("qt_appearance_provider.doc/parse_font.md")]
fn parse_font(value: &str) -> Option<SystemFont> {
    // Qt writes a font as a comma-separated record whose first two fields are
    // the family and the point size; the rest is Qt's own business.
    let mut parts = value.split(',');
    let family = parts.next()?.trim();
    if family.is_empty() {
        return None;
    }
    let size = parts
        .next()
        .and_then(|size| size.trim().parse::<f32>().ok())
        .filter(|size| *size > 0.0);
    Some(SystemFont::new(family, size))
}

#[doc = include_str!("qt_appearance_provider.doc/luminance.md")]
fn luminance(color: Color) -> f32 {
    // Rec. 601 luma. Enough to tell a dark surface from a light one, which is
    // the only question asked of it.
    0.299 * color.r + 0.587 * color.g + 0.114 * color.b
}

#[cfg(test)]
mod tests {
    use super::*;

    const BREEZE_DARK: &str = "\
[General]
ColorScheme=BreezeDark
AccentColor=61,174,233
font=Noto Sans,10,-1,5,50,0,0,0,0,0
fixed=Hack,10,-1,5,50,0,0,0,0,0

[Colors:Window]
BackgroundNormal=42,46,50
ForegroundNormal=252,252,252

[Colors:View]
BackgroundNormal=27,30,32
ForegroundNormal=252,252,252

[Colors:Selection]
BackgroundNormal=61,174,233
ForegroundNormal=252,252,252

[Icons]
Theme=breeze-dark

[KDE]
AnimationDurationFactor=1
";

    #[test]
    fn a_kde_config_yields_the_whole_shape() {
        let seen = parse(BREEZE_DARK);
        assert_eq!(seen.color_scheme, Some(ColorScheme::Dark));
        assert_eq!(
            seen.palette.stated(),
            6,
            "Qt is the source that has colours"
        );
        assert_eq!(seen.icon_theme.as_deref(), Some("breeze-dark"));
        assert_eq!(seen.reduced_motion, Some(false));
    }

    #[test]
    fn the_scheme_is_measured_rather_than_read_off_the_name() {
        // A dark palette under a theme whose name says nothing. Trusting the
        // name here would answer Light for a black window.
        let unnamed = BREEZE_DARK.replace("ColorScheme=BreezeDark", "ColorScheme=Midnight");
        assert_eq!(parse(&unnamed).color_scheme, Some(ColorScheme::Dark));
    }

    #[test]
    fn the_name_is_the_fallback_when_there_is_no_palette() {
        let name_only = "[General]\nColorScheme=BreezeDark\n";
        let seen = parse(name_only);
        assert_eq!(seen.color_scheme, Some(ColorScheme::Dark));
        assert!(seen.palette.is_empty());
    }

    #[test]
    fn a_font_record_gives_up_its_family_and_size_and_ignores_the_rest() {
        let font = parse(BREEZE_DARK).font.expect("a font");
        assert_eq!(&*font.family, "Noto Sans");
        assert_eq!(font.size_pt, Some(10.0));
        // 10pt is 13.33px, and it is not rounded here.
        assert!((font.size_px().unwrap() - 13.333).abs() < 0.01);
    }

    #[test]
    fn an_rgb_triple_becomes_a_colour_and_a_malformed_one_becomes_nothing() {
        assert_eq!(parse_rgb("255,0,0"), Some(Color::rgb(1.0, 0.0, 0.0)));
        assert_eq!(parse_rgb("0,0,0,128").map(|c| c.a), Some(128.0 / 255.0));
        assert_eq!(parse_rgb("255,0"), None, "two components is not a colour");
        assert_eq!(parse_rgb("1,2,3,4,5"), None, "nor is five");
        assert_eq!(parse_rgb("#ff0000"), None, "this format is not hex");
        assert_eq!(parse_rgb("300,0,0"), None, "and 300 does not fit a byte");
    }

    #[test]
    fn an_absent_file_answers_nothing_and_says_so() {
        let provider = QtAppearanceProvider::at("/nonexistent/kdeglobals");
        assert!(provider.appearance().is_empty());
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(!provider.answers());
    }

    #[test]
    fn a_zero_duration_factor_is_a_request_for_less_motion() {
        let still = BREEZE_DARK.replace("AnimationDurationFactor=1", "AnimationDurationFactor=0");
        assert_eq!(parse(&still).reduced_motion, Some(true));
    }

    #[test]
    fn comments_and_blank_lines_are_not_settings() {
        let noisy = "# a comment\n\n[General]\n; another\nColorScheme=BreezeDark\n";
        assert_eq!(parse(noisy).color_scheme, Some(ColorScheme::Dark));
    }

    #[test]
    fn a_key_outside_any_section_does_not_become_one_from_the_section_before() {
        // Two sections with the same key name is the whole reason the map is
        // keyed by the pair.
        let two = "[Colors:Window]\nBackgroundNormal=0,0,0\n[Colors:View]\nBackgroundNormal=255,255,255\n";
        let seen = parse(two);
        assert_eq!(
            seen.palette.window_background,
            Some(Color::rgb(0.0, 0.0, 0.0))
        );
        assert_eq!(
            seen.palette.view_background,
            Some(Color::rgb(1.0, 1.0, 1.0))
        );
    }

    #[test]
    fn an_empty_file_answers_nothing_rather_than_defaults() {
        assert!(parse("").is_empty());
    }
}

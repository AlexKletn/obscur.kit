//! Reading appearance from GNOME's interface settings.

use std::process::Command;

use obscur_capability::SupportLevel;
use obscur_scene::color::Color;
use obscur_system::appearance::{
    AppearanceCallback, AppearanceProvider, AppearanceWatch, ColorScheme, SystemAppearance,
    SystemFont, WindowButton, WindowButtonIcons, WindowButtonLayout, WindowDecoration,
};

use super::monitor_watch::MonitorWatch;

/// The settings schema every key here comes from.
const SCHEMA: &str = "org.gnome.desktop.interface";
const WINDOW_SCHEMA: &str = "org.gnome.desktop.wm.preferences";
const BUTTON_LAYOUT: &str = "button-layout";

#[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/GtkAppearanceProvider.md")]
#[derive(Debug, Clone)]
pub struct GtkAppearanceProvider {
    tool: Box<str>,
}

impl GtkAppearanceProvider {
    /// Read through `gsettings`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_tool("gsettings")
    }

    /// Read through a nominated tool, for a session that renames it, and for tests.
    #[must_use]
    pub fn with_tool(tool: impl Into<Box<str>>) -> Self {
        Self { tool: tool.into() }
    }

    /// The tool this provider shells out to.
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    #[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/read.md")]
    fn read(&self, key: &str) -> Option<String> {
        self.read_setting(SCHEMA, key)
            .filter(|value| !value.is_empty())
    }

    fn read_setting(&self, schema: &str, key: &str) -> Option<String> {
        let output = Command::new(&*self.tool)
            .args(["get", schema, key])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        Some(unquote(text.trim()).to_owned())
    }
}

impl Default for GtkAppearanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AppearanceProvider for GtkAppearanceProvider {
    fn appearance(&self) -> SystemAppearance {
        SystemAppearance {
            color_scheme: self.read("color-scheme").and_then(|v| parse_scheme(&v)),
            contrast: None,
            accent: self.read("accent-color").and_then(|v| accent_hex(&v)),
            // Deliberately empty, and the single most important honest answer in
            // this file: `gsettings` names the theme, and the theme's colours
            // live inside GTK. Reading them means linking GTK, which this
            // adapter does not do, so it says nothing rather than guessing.
            palette: obscur_system::appearance::SystemPalette::default(),
            window_radius: self
                .read("gtk-theme")
                .as_deref()
                .and_then(super::adwaita_theme::window_radius),
            window_decoration: self
                .read_setting(WINDOW_SCHEMA, BUTTON_LAYOUT)
                .as_deref()
                .and_then(parse_decoration),
            font: self.read("font-name").and_then(|v| parse_font(&v)),
            monospace_font: self
                .read("monospace-font-name")
                .and_then(|v| parse_font(&v)),
            text_scale: self
                .read("text-scaling-factor")
                .and_then(|v| v.parse::<f32>().ok())
                .filter(|scale| *scale > 0.0),
            cursor_size: self.read("cursor-size").and_then(|v| v.parse::<u32>().ok()),
            icon_theme: self.read("icon-theme").map(Into::into),
            reduced_motion: self
                .read("enable-animations")
                .and_then(|v| v.parse::<bool>().ok())
                .map(|enabled| !enabled),
        }
    }

    fn support(&self) -> SupportLevel {
        if !tool_available(&self.tool) {
            return SupportLevel::Unavailable;
        }
        // Limited, always, and it is not modesty: this source cannot state a
        // palette at any version, so a caller that reads `Native` here and then
        // finds no colours would have been misled.
        SupportLevel::Limited
    }

    #[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        if tool_available(&self.tool) {
            // `gsettings monitor` streams a line per changed key for the whole
            // schema, which covers every setting this source reads.
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }

    /// Subscribe through `gsettings monitor` on the interface schema and the window
    /// manager's button-layout key. Dropping the returned guard stops both monitors.
    fn watch(&self, on_change: AppearanceCallback) -> Option<AppearanceWatch> {
        let interface =
            MonitorWatch::spawn(&self.tool, &["monitor", SCHEMA], ":", on_change.clone());
        let layout = MonitorWatch::spawn(
            &self.tool,
            &["monitor", WINDOW_SCHEMA, BUTTON_LAYOUT],
            ":",
            on_change,
        );
        if interface.is_none() && layout.is_none() {
            return None;
        }
        Some(AppearanceWatch::new(move || drop((interface, layout))))
    }
}

fn parse_decoration(value: &str) -> Option<WindowDecoration> {
    fn buttons(group: &str) -> Vec<WindowButton> {
        group
            .split(',')
            .filter_map(|name| {
                Some(match name.trim() {
                    "menu" => WindowButton::WindowMenu,
                    "appmenu" => WindowButton::ApplicationMenu,
                    "icon" => WindowButton::WindowIcon,
                    "minimize" => WindowButton::Minimize,
                    "maximize" => WindowButton::Maximize,
                    "close" => WindowButton::Close,
                    "spacer" => WindowButton::Spacer,
                    _ => return None,
                })
            })
            .collect()
    }
    let (left, right) = value.split_once(':').unwrap_or((value, ""));
    if right.contains(':') {
        return None;
    }
    Some(WindowDecoration {
        button_layout: WindowButtonLayout {
            left: Some(buttons(left)),
            right: Some(buttons(right)),
        },
        button_icons: WindowButtonIcons {
            minimize: Some("window-minimize-symbolic".into()),
            maximize: Some("window-maximize-symbolic".into()),
            restore: Some("window-restore-symbolic".into()),
            close: Some("window-close-symbolic".into()),
        },
    })
}

/// `gsettings` prints strings quoted; numbers and booleans it does not.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('\'')
        .and_then(|v| v.strip_suffix('\''))
        .or_else(|| value.strip_prefix('"').and_then(|v| v.strip_suffix('"')))
        .unwrap_or(value)
}

#[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/parse_scheme.md")]
fn parse_scheme(value: &str) -> Option<ColorScheme> {
    match value {
        "prefer-dark" => Some(ColorScheme::Dark),
        "prefer-light" => Some(ColorScheme::Light),
        // `default` is GNOME saying the user expressed no preference, which is
        // the absence of an answer and not a light one.
        _ => None,
    }
}

#[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/parse_font.md")]
fn parse_font(value: &str) -> Option<SystemFont> {
    // Pango writes "Family Style Size", and only the trailing size is
    // unambiguous: everything before it is the family and its style words,
    // which this does not try to separate.
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match value.rsplit_once(' ') {
        Some((family, size)) => match size.parse::<f32>() {
            Ok(size) if size > 0.0 => Some(SystemFont::new(family.trim(), Some(size))),
            _ => Some(SystemFont::new(value, None)),
        },
        None => Some(SystemFont::new(value, None)),
    }
}

#[doc = include_str!("../../docs/api/provider/gtk_appearance_provider.doc/accent_hex.md")]
fn accent_hex(name: &str) -> Option<Color> {
    // GNOME 47 states the accent as one of nine NAMES, not as a colour. These
    // are the values Adwaita publishes for them, so this is a lookup of the
    // desktop's own palette rather than a choice made here — and a name that is
    // not on the list answers nothing rather than the nearest guess.
    let hex = match name {
        "blue" => 0x35_84_E4,
        "teal" => 0x21_90_A4,
        "green" => 0x3A_94_4A,
        "yellow" => 0xC8_88_00,
        "orange" => 0xED_5B_00,
        "red" => 0xE6_2D_42,
        "pink" => 0xD5_61_99,
        "purple" => 0x91_41_AC,
        "slate" => 0x6F_83_96,
        _ => return None,
    };
    Some(from_hex(hex))
}

/// A packed `0xRRGGBB` as a colour.
fn from_hex(hex: u32) -> Color {
    #[allow(clippy::cast_precision_loss)]
    Color::rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

/// Whether the tool is on `PATH` at all.
fn tool_available(tool: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {tool}")])
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoration_preserves_button_order_and_has_a_restore_icon() {
        let decoration = parse_decoration("icon:close,maximize,minimize").unwrap();
        assert_eq!(
            decoration.button_layout.left,
            Some(vec![WindowButton::WindowIcon])
        );
        assert_eq!(
            decoration.button_layout.right,
            Some(vec![
                WindowButton::Close,
                WindowButton::Maximize,
                WindowButton::Minimize,
            ])
        );
        assert_eq!(
            decoration.button_icons.restore.as_deref(),
            Some("window-restore-symbolic")
        );
        assert_eq!(
            parse_decoration(":").unwrap().button_layout.right,
            Some(vec![])
        );
    }

    #[test]
    fn the_default_tool_is_gsettings() {
        assert_eq!(GtkAppearanceProvider::new().tool(), "gsettings");
    }

    #[test]
    fn a_missing_tool_answers_nothing_and_says_so() {
        let provider = GtkAppearanceProvider::with_tool("obscur-no-such-tool");
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(!provider.answers());
        assert!(provider.appearance().is_empty());
    }

    #[test]
    fn gsettings_quoting_is_removed() {
        assert_eq!(unquote("'prefer-dark'"), "prefer-dark");
        assert_eq!(unquote("\"Adwaita\""), "Adwaita");
        assert_eq!(unquote("24"), "24", "an unquoted value is left alone");
    }

    #[test]
    fn no_preference_is_no_answer_rather_than_light() {
        // GNOME's `default` means the user did not choose, and answering Light
        // for it would put a light interface on a user who set none.
        assert_eq!(parse_scheme("prefer-dark"), Some(ColorScheme::Dark));
        assert_eq!(parse_scheme("prefer-light"), Some(ColorScheme::Light));
        assert_eq!(parse_scheme("default"), None);
        assert_eq!(parse_scheme(""), None);
    }

    #[test]
    fn a_pango_description_splits_at_its_trailing_size() {
        let font = parse_font("Cantarell 11").expect("a font");
        assert_eq!(&*font.family, "Cantarell");
        assert_eq!(font.size_pt, Some(11.0));
    }

    #[test]
    fn a_multi_word_family_keeps_its_words() {
        let font = parse_font("Noto Sans Mono 10").expect("a font");
        assert_eq!(&*font.family, "Noto Sans Mono");
        assert_eq!(font.size_pt, Some(10.0));
    }

    #[test]
    fn a_family_with_no_size_is_still_a_family() {
        // Nothing to split on, and inventing 10pt would be indistinguishable
        // from the user having chosen it.
        let font = parse_font("Cantarell").expect("a font");
        assert_eq!(&*font.family, "Cantarell");
        assert_eq!(font.size_pt, None);
        assert_eq!(parse_font("   "), None);
    }

    #[test]
    fn a_style_word_before_the_size_stays_with_the_family() {
        // Pango puts style words in the same string, and separating them needs
        // Pango's own table. Keeping them is honest; guessing is not.
        let font = parse_font("Cantarell Bold 11").expect("a font");
        assert_eq!(&*font.family, "Cantarell Bold");
    }

    #[test]
    fn every_gnome_accent_name_resolves_and_nothing_else_does() {
        for name in [
            "blue", "teal", "green", "yellow", "orange", "red", "pink", "purple", "slate",
        ] {
            assert!(accent_hex(name).is_some(), "{name} is a GNOME accent");
        }
        assert_eq!(accent_hex("chartreuse"), None);
        assert_eq!(accent_hex("#3584e4"), None, "a hex string is not a name");
    }

    #[test]
    fn the_blue_accent_is_adwaitas_blue() {
        let blue = accent_hex("blue").expect("blue");
        assert!((blue.r - 0.208).abs() < 0.01, "{blue:?}");
        assert!((blue.g - 0.518).abs() < 0.01, "{blue:?}");
        assert!((blue.b - 0.894).abs() < 0.01, "{blue:?}");
    }

    #[test]
    fn this_source_never_claims_a_palette() {
        // The claim the whole file turns on. If this ever starts answering
        // colours, `support` must stop saying `Limited`.
        let provider = GtkAppearanceProvider::new();
        assert!(provider.appearance().palette.is_empty());
        assert_ne!(provider.support(), SupportLevel::Native);
    }
}

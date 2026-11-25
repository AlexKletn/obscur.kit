//! Reading appearance from `xdg-desktop-portal`.

use std::process::Command;

use obscur_capability::SupportLevel;
use obscur_scene::color::Color;
use obscur_system::appearance::{
    AppearanceCallback, AppearanceProvider, AppearanceWatch, ColorScheme, Contrast,
    SystemAppearance, SystemPalette,
};

use super::monitor_watch::MonitorWatch;

/// The portal settings namespace all three keys live in.
const NAMESPACE: &str = "org.freedesktop.appearance";

#[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/PortalAppearanceProvider.md")]
#[derive(Debug, Clone)]
pub struct PortalAppearanceProvider {
    tool: Box<str>,
}

impl PortalAppearanceProvider {
    /// Read through `gdbus`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_tool("gdbus")
    }

    /// Read through a nominated tool, and for tests.
    #[must_use]
    pub fn with_tool(tool: impl Into<Box<str>>) -> Self {
        Self { tool: tool.into() }
    }

    /// The tool this provider shells out to.
    #[must_use]
    pub fn tool(&self) -> &str {
        &self.tool
    }

    /// One `Settings.ReadAll` call for the namespace, or nothing.
    fn read_all(&self) -> Option<String> {
        let output = Command::new(&*self.tool)
            .args([
                "call",
                "--session",
                "--dest",
                "org.freedesktop.portal.Desktop",
                "--object-path",
                "/org/freedesktop/portal/desktop",
                "--method",
                "org.freedesktop.portal.Settings.ReadAll",
                &format!("['{NAMESPACE}']"),
            ])
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8(output.stdout).ok())
            .flatten()
    }
}

impl Default for PortalAppearanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AppearanceProvider for PortalAppearanceProvider {
    fn appearance(&self) -> SystemAppearance {
        self.read_all()
            .map_or_else(SystemAppearance::default, |text| parse(&text))
    }

    fn support(&self) -> SupportLevel {
        if !tool_available(&self.tool) {
            return SupportLevel::Unavailable;
        }
        // Three keys, and no font, cursor or icon theme among them: the portal
        // is the most portable source and the narrowest.
        SupportLevel::Limited
    }

    #[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/watch_support.md")]
    fn watch_support(&self) -> SupportLevel {
        if tool_available(&self.tool) {
            // `org.freedesktop.portal.Settings` declares a `SettingChanged`
            // signal and emits it per key. This is the source that was designed
            // to be watched, and the only one here that was.
            SupportLevel::Native
        } else {
            SupportLevel::Unavailable
        }
    }

    #[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/watch.md")]
    fn watch(&self, on_change: AppearanceCallback) -> Option<AppearanceWatch> {
        MonitorWatch::spawn(
            &self.tool,
            &[
                "monitor",
                "--session",
                "--dest",
                "org.freedesktop.portal.Desktop",
                "--object-path",
                "/org/freedesktop/portal/desktop",
            ],
            NAMESPACE,
            on_change,
        )
    }
}

#[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/parse.md")]
fn parse(text: &str) -> SystemAppearance {
    SystemAppearance {
        color_scheme: read_u32(text, "color-scheme").and_then(scheme_of),
        contrast: read_u32(text, "contrast").map(|value| {
            if value == 1 {
                Contrast::High
            } else {
                Contrast::Normal
            }
        }),
        accent: read_accent(text),
        palette: SystemPalette::default(),
        font: None,
        monospace_font: None,
        text_scale: None,
        cursor_size: None,
        icon_theme: None,
        window_radius: None,
        window_decoration: None,
        reduced_motion: None,
    }
}

#[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/scheme_of.md")]
const fn scheme_of(value: u32) -> Option<ColorScheme> {
    // The portal's own numbering: 0 no preference, 1 dark, 2 light. Zero is the
    // absence of an answer, so it stays absent rather than becoming light.
    match value {
        1 => Some(ColorScheme::Dark),
        2 => Some(ColorScheme::Light),
        _ => None,
    }
}

/// The `<uint32 N>` that follows a named key.
fn read_u32(text: &str, key: &str) -> Option<u32> {
    let at = text.find(&format!("'{key}'"))?;
    let rest = &text[at..];
    let open = rest.find('<')?;
    let close = rest[open..].find('>')? + open;
    rest[open + 1..close]
        .trim()
        .trim_start_matches("uint32")
        .trim()
        .parse()
        .ok()
}

#[doc = include_str!("../../docs/api/provider/portal_appearance_provider.doc/read_accent.md")]
fn read_accent(text: &str) -> Option<Color> {
    let at = text.find("'accent-color'")?;
    let rest = &text[at..];
    let open = rest.find('(')?;
    let close = rest[open..].find(')')? + open;
    let mut parts = rest[open + 1..close]
        .split(',')
        .map(|part| part.trim().parse::<f32>());
    let r = parts.next()?.ok()?;
    let g = parts.next()?.ok()?;
    let b = parts.next()?.ok()?;
    if parts.next().is_some() {
        return None;
    }
    // The portal states components as doubles in 0..1, and a value outside that
    // is not the thing this is parsing.
    let in_range = |v: f32| (0.0..=1.0).contains(&v);
    (in_range(r) && in_range(g) && in_range(b)).then(|| Color::rgb(r, g, b))
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

    /// What `gdbus call … Settings.ReadAll` prints on a desktop that answers
    /// all three keys. Kept verbatim, because the parsing below is only worth
    /// anything against the real shape.
    const FULL: &str = "({'org.freedesktop.appearance': {'color-scheme': <uint32 1>, \
                        'accent-color': <(0.20784313725490197, 0.5176470588235295, \
                        0.8941176470588236)>, 'contrast': <uint32 0>}},)\n";

    #[test]
    fn a_full_answer_is_read() {
        let seen = parse(FULL);
        assert_eq!(seen.color_scheme, Some(ColorScheme::Dark));
        assert_eq!(seen.contrast, Some(Contrast::Normal));
        let accent = seen.accent.expect("an accent");
        assert!((accent.r - 0.2078).abs() < 0.001, "{accent:?}");
        assert!((accent.b - 0.8941).abs() < 0.001, "{accent:?}");
    }

    #[test]
    fn the_portal_states_no_palette_and_no_font() {
        let seen = parse(FULL);
        assert!(seen.palette.is_empty());
        assert_eq!(seen.font, None);
        assert_eq!(seen.icon_theme, None);
    }

    #[test]
    fn no_preference_stays_absent() {
        let none = FULL.replace("'color-scheme': <uint32 1>", "'color-scheme': <uint32 0>");
        assert_eq!(parse(&none).color_scheme, None);
    }

    #[test]
    fn light_is_two_and_dark_is_one() {
        // The portal's numbering is not the obvious one and getting it backwards
        // would be invisible in a screenshot of the wrong theme.
        assert_eq!(scheme_of(1), Some(ColorScheme::Dark));
        assert_eq!(scheme_of(2), Some(ColorScheme::Light));
        assert_eq!(scheme_of(0), None);
        assert_eq!(scheme_of(7), None);
    }

    #[test]
    fn high_contrast_is_read() {
        let high = FULL.replace("'contrast': <uint32 0>", "'contrast': <uint32 1>");
        assert_eq!(parse(&high).contrast, Some(Contrast::High));
    }

    #[test]
    fn a_desktop_that_answers_only_the_scheme_answers_only_the_scheme() {
        let partial = "({'org.freedesktop.appearance': {'color-scheme': <uint32 2>}},)\n";
        let seen = parse(partial);
        assert_eq!(seen.color_scheme, Some(ColorScheme::Light));
        assert_eq!(seen.accent, None);
        assert_eq!(seen.contrast, None);
    }

    #[test]
    fn an_empty_or_unrelated_reply_yields_nothing() {
        assert!(parse("").is_empty());
        assert!(parse("({},)").is_empty());
        assert!(parse("nonsense").is_empty());
    }

    #[test]
    fn an_accent_outside_the_unit_range_is_refused() {
        // 0..255 would parse as numbers and paint a white accent; the portal
        // states doubles, so anything else is not this format.
        let bytes = FULL.replace(
            "<(0.20784313725490197, 0.5176470588235295, 0.8941176470588236)>",
            "<(53, 132, 228)>",
        );
        assert_eq!(parse(&bytes).accent, None);
    }

    /// Captured verbatim from a real desktop (Hyprland, `xdg-desktop-portal`
    /// Settings v2) on 2026-08-08. Two things a constructed sample would not
    /// have caught: the keys arrive in a different order, and `accent-color` is
    /// simply absent rather than empty.
    const REAL: &str =
        "({'org.freedesktop.appearance': {'contrast': <uint32 0>, 'color-scheme': <uint32 2>}},)\n";

    #[test]
    fn a_real_desktops_reply_reads_the_same_whatever_order_the_keys_arrive_in() {
        let seen = parse(REAL);
        assert_eq!(seen.color_scheme, Some(ColorScheme::Light));
        assert_eq!(seen.contrast, Some(Contrast::Normal));
        assert_eq!(
            seen.accent, None,
            "this desktop states no accent, and absent must stay absent"
        );
    }

    #[test]
    fn a_missing_tool_answers_nothing_and_says_so() {
        let provider = PortalAppearanceProvider::with_tool("obscur-no-such-tool");
        assert_eq!(provider.support(), SupportLevel::Unavailable);
        assert!(provider.appearance().is_empty());
    }

    #[test]
    fn the_default_tool_is_gdbus() {
        assert_eq!(PortalAppearanceProvider::new().tool(), "gdbus");
    }
}

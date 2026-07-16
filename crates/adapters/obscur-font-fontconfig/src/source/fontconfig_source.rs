//! The [`FontconfigSource`]: system font lookup by shelling out to fontconfig's
//! CLI (spec §5.18).

use std::path::Path;
use std::process::Command;

use obscur_text::font::{FontAttributes, FontStretch, GenericFamily};
use obscur_text::span::{FontSlant, FontWeight};

use super::attribute_scale::{css_slant, css_stretch, css_weight};
use super::system_face::SystemFace;

/// The record separator asked of fontconfig's `-f` format. Font paths and family
/// names may contain spaces and commas but not tabs in any real installation.
const FIELD: char = '\t';

/// The `-f` format both queries request: family, file, weight, slant, width.
const FORMAT: &str = "%{family[0]}\t%{file}\t%{weight}\t%{slant}\t%{width}\n";

/// Locates host-installed faces through fontconfig (spec §5.18).
///
/// The tool names are held rather than hard-coded so a caller may point at a
/// different fontconfig build; the defaults are the conventional `fc-match` /
/// `fc-list`.
#[derive(Debug, Clone)]
pub struct FontconfigSource {
    match_tool: Box<str>,
    list_tool: Box<str>,
}

impl FontconfigSource {
    /// Creates a source backed by the conventional `fc-match` / `fc-list`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            match_tool: Box::from("fc-match"),
            list_tool: Box::from("fc-list"),
        }
    }

    /// Creates a source backed by explicitly named tools.
    #[must_use]
    pub fn with_tools(match_tool: impl Into<Box<str>>, list_tool: impl Into<Box<str>>) -> Self {
        Self {
            match_tool: match_tool.into(),
            list_tool: list_tool.into(),
        }
    }

    /// Whether fontconfig is usable here — i.e. both tools ran.
    ///
    /// A host with no fontconfig (a bare container, a non-Linux target, a
    /// stripped image) is a normal state, not an error: the caller is expected to
    /// supply bytes some other way.
    #[must_use]
    pub fn is_available(&self) -> bool {
        Self::run(&self.match_tool, &["--version"]).is_some()
            && Self::run(&self.list_tool, &["--version"]).is_some()
    }

    /// Resolves a CSS **generic** family to the face the host nominates for it.
    ///
    /// This is `fc-match`, and substitution is the point: `sans-serif` is a
    /// request for *whatever this host calls its sans*, so an answer that is not
    /// literally named "sans-serif" is correct rather than a miss. Returns `None`
    /// only when fontconfig is absent, answers nothing, or answers with a file
    /// this toolkit cannot load (see [`is_loadable`]).
    #[must_use]
    pub fn match_generic(&self, generic: GenericFamily) -> Option<SystemFace> {
        let pattern = generic_pattern(generic);
        let out = Self::run(&self.match_tool, &["-f", FORMAT, pattern])?;
        parse_faces(&out).into_iter().next()
    }

    /// Finds an installed face by its **named** family, or `None` when the host
    /// does not have it.
    ///
    /// # Why not `fc-match`
    ///
    /// `fc-match` never fails: asked for a family that is not installed it returns
    /// the system default with exit status 0. Built on it, this would report every
    /// absent family as present under a substituted face — `font-family: Inter,
    /// sans-serif` would resolve "Inter" to Noto Sans on a machine with no Inter,
    /// and the author's fallback chain would never be walked.
    ///
    /// Two things prevent that, and they are **redundant** — either alone suffices,
    /// so neither should be read as the sole guard: `fc-list` returns nothing for
    /// an absent family, and the name filter below rejects a face whose family is
    /// not the one asked for. The filter is what makes the guarantee independent of
    /// which tool is wired; `fc-list` is what leaves it nothing to reject.
    ///
    /// The independent reason to prefer `fc-list` is **attribute choice**: it lists
    /// every installed cut, so the nearest to `attributes` by
    /// [`FontAttributes::distance`] is picked here. `fc-match` returns a single
    /// face, which would make that fontconfig's decision rather than this
    /// toolkit's.
    #[must_use]
    pub fn find_family(&self, name: &str, attributes: &FontAttributes) -> Option<SystemFace> {
        let out = Self::run(&self.list_tool, &["-f", FORMAT, name])?;
        parse_faces(&out)
            .into_iter()
            .filter(|face| face.family().eq_ignore_ascii_case(name))
            .min_by_key(|face| face.attributes().distance(attributes))
    }

    /// Runs `tool` with `args`, returning its stdout on success.
    ///
    /// Any failure — tool absent, non-zero status, non-UTF-8 output — collapses to
    /// `None`. There is nothing a caller could do with the distinction that it
    /// would not also do for "no fonts here".
    fn run(tool: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(tool).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        String::from_utf8(out.stdout).ok()
    }
}

impl Default for FontconfigSource {
    /// The default source uses `fc-match` / `fc-list`.
    fn default() -> Self {
        Self::new()
    }
}

/// The fontconfig pattern naming a CSS generic family.
///
/// fontconfig's own config defines `serif` / `sans-serif` / `monospace` aliases;
/// `cursive` and `fantasy` are CSS generics fontconfig also recognises. `SystemUi`
/// has no fontconfig alias, so it asks for the same face `sans-serif` does —
/// which is what the freedesktop stack means by "the UI font".
const fn generic_pattern(generic: GenericFamily) -> &'static str {
    match generic {
        GenericFamily::Serif => "serif",
        GenericFamily::SansSerif | GenericFamily::SystemUi => "sans-serif",
        GenericFamily::Monospace => "monospace",
        GenericFamily::Cursive => "cursive",
        GenericFamily::Fantasy => "fantasy",
    }
}

/// Whether this toolkit can actually load `path`'s bytes as a single face.
///
/// **TrueType/OpenType collections (`.ttc`) are excluded**, and this is a real
/// exclusion rather than caution: the shaping stack parses faces with
/// `read_fonts::FontRef::new`, which rejects a collection outright
/// (`InvalidSfnt(0x74746366)` — the `ttcf` tag) because a collection needs a face
/// index this format string never carries. A `.ttc` handed onward would register
/// as a face with no readable metrics and no `cmap` coverage, so it would resolve
/// nothing and its text would silently *vanish*. Dropping it here instead lets the
/// caller fall through to a face that works.
///
/// The cost is real and worth naming: on a host whose only face for a script is a
/// collection — CJK installations commonly ship `.ttc` — this reports no face at
/// all rather than a broken one. Collection support is unbuilt.
fn is_loadable(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("ttf" | "otf"))
}

/// Parses `FORMAT`-shaped fontconfig output into faces, dropping any line that is
/// malformed or names a file this toolkit cannot load.
fn parse_faces(out: &str) -> Vec<SystemFace> {
    out.lines().filter_map(parse_face).collect()
}

/// Parses one `family\tfile\tweight\tslant\twidth` line.
fn parse_face(line: &str) -> Option<SystemFace> {
    let mut fields = line.split(FIELD);
    let family = fields.next()?.trim();
    let file = fields.next()?.trim();
    if family.is_empty() || file.is_empty() {
        return None;
    }
    let path = Path::new(file);
    if !is_loadable(path) {
        return None;
    }
    // A face whose numeric fields are missing or unreadable is still a usable
    // face; it just reads as the default cut rather than being dropped.
    let weight = fields.next().and_then(|f| f.trim().parse::<i32>().ok());
    let slant = fields.next().and_then(|f| f.trim().parse::<i32>().ok());
    let width = fields.next().and_then(|f| f.trim().parse::<i32>().ok());
    let attributes = FontAttributes::new()
        .with_weight(weight.map_or_else(FontWeight::default, css_weight))
        .with_slant(slant.map_or_else(FontSlant::default, css_slant))
        .with_stretch(width.map_or_else(FontStretch::default, css_stretch));
    Some(SystemFace::new(family, path, attributes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_text::span::{FontSlant, FontWeight};

    #[test]
    fn parses_a_face_line_and_converts_the_scales() {
        let faces = parse_faces("Noto Sans\t/f/NotoSans-Bold.ttf\t200\t100\t100\n");
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].family(), "Noto Sans");
        assert_eq!(faces[0].path(), Path::new("/f/NotoSans-Bold.ttf"));
        assert_eq!(faces[0].attributes().weight, FontWeight::BOLD);
        assert_eq!(faces[0].attributes().slant, FontSlant::Italic);
    }

    /// A `.ttc` must never reach the registry — see [`is_loadable`]. This is the
    /// difference between falling through to a working face and text vanishing.
    #[test]
    fn collections_are_dropped_rather_than_registered_unloadable() {
        let out = "Noto Sans CJK\t/f/NotoSansCJK-Regular.ttc\t80\t0\t100\n\
                   Noto Sans\t/f/NotoSans-Regular.ttf\t80\t0\t100\n";
        let faces = parse_faces(out);
        assert_eq!(faces.len(), 1, "the .ttc line must be dropped");
        assert_eq!(faces[0].path(), Path::new("/f/NotoSans-Regular.ttf"));
    }

    #[test]
    fn malformed_lines_are_skipped_not_fatal() {
        assert!(parse_faces("garbage\n\n\t\n").is_empty());
    }

    #[test]
    fn a_face_with_unreadable_numeric_fields_reads_as_the_default_cut() {
        let faces = parse_faces("Fam\t/f/x.ttf\tnot-a-number\t\t\n");
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].attributes(), &FontAttributes::new());
    }

    #[test]
    fn generic_patterns_are_the_fontconfig_aliases() {
        assert_eq!(generic_pattern(GenericFamily::Monospace), "monospace");
        assert_eq!(generic_pattern(GenericFamily::SansSerif), "sans-serif");
        // No fontconfig alias exists for `system-ui`; it shares the sans answer.
        assert_eq!(generic_pattern(GenericFamily::SystemUi), "sans-serif");
    }
}

#![doc = include_str!("fontconfig_source.doc/_module.md")]

use std::path::Path;
use std::process::Command;

use obscur_text::font::{FontAttributes, FontStretch, GenericFamily};
use obscur_text::span::{FontSlant, FontWeight};

use super::attribute_scale::{css_slant, css_stretch, css_weight};
use super::system_face::SystemFace;

#[doc = include_str!("fontconfig_source.doc/FIELD.md")]
const FIELD: char = '\t';

#[doc = include_str!("fontconfig_source.doc/FORMAT.md")]
const FORMAT: &str = "%{family[0]}\t%{file}\t%{weight}\t%{slant}\t%{width}\n";

#[doc = include_str!("fontconfig_source.doc/FontconfigSource.md")]
#[derive(Debug, Clone)]
pub struct FontconfigSource {
    match_tool: Box<str>,
    list_tool: Box<str>,
}

impl FontconfigSource {
    #[doc = include_str!("fontconfig_source.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            match_tool: Box::from("fc-match"),
            list_tool: Box::from("fc-list"),
        }
    }

    #[doc = include_str!("fontconfig_source.doc/with_tools.md")]
    #[must_use]
    pub fn with_tools(match_tool: impl Into<Box<str>>, list_tool: impl Into<Box<str>>) -> Self {
        Self {
            match_tool: match_tool.into(),
            list_tool: list_tool.into(),
        }
    }

    #[doc = include_str!("fontconfig_source.doc/is_available.md")]
    #[must_use]
    pub fn is_available(&self) -> bool {
        Self::run(&self.match_tool, &["--version"]).is_some()
            && Self::run(&self.list_tool, &["--version"]).is_some()
    }

    #[doc = include_str!("fontconfig_source.doc/match_generic.md")]
    #[must_use]
    pub fn match_generic(&self, generic: GenericFamily) -> Option<SystemFace> {
        let pattern = generic_pattern(generic);
        let out = Self::run(&self.match_tool, &["-f", FORMAT, pattern])?;
        parse_faces(&out).into_iter().next()
    }

    #[doc = include_str!("fontconfig_source.doc/match_char.md")]
    #[must_use]
    pub fn match_char(&self, character: char) -> Option<SystemFace> {
        let pattern = format!(":charset={:x}", character as u32);
        let out = Self::run(&self.match_tool, &["-f", FORMAT, &pattern])?;
        parse_faces(&out).into_iter().next()
    }

    #[doc = include_str!("fontconfig_source.doc/find_family.md")]
    #[must_use]
    pub fn find_family(&self, name: &str, attributes: &FontAttributes) -> Option<SystemFace> {
        let out = Self::run(&self.list_tool, &["-f", FORMAT, name])?;
        parse_faces(&out)
            .into_iter()
            .filter(|face| face.family().eq_ignore_ascii_case(name))
            .min_by_key(|face| face.attributes().distance(attributes))
    }

    #[doc = include_str!("fontconfig_source.doc/run.md")]
    fn run(tool: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(tool).args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        String::from_utf8(out.stdout).ok()
    }
}

impl Default for FontconfigSource {
    #[doc = include_str!("fontconfig_source.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

#[doc = include_str!("fontconfig_source.doc/generic_pattern.md")]
const fn generic_pattern(generic: GenericFamily) -> &'static str {
    match generic {
        GenericFamily::Serif => "serif",
        GenericFamily::SansSerif | GenericFamily::SystemUi => "sans-serif",
        GenericFamily::Monospace => "monospace",
        GenericFamily::Cursive => "cursive",
        GenericFamily::Fantasy => "fantasy",
    }
}

#[doc = include_str!("fontconfig_source.doc/is_loadable.md")]
fn is_loadable(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("ttf" | "otf"))
}

#[doc = include_str!("fontconfig_source.doc/parse_faces.md")]
fn parse_faces(out: &str) -> Vec<SystemFace> {
    out.lines().filter_map(parse_face).collect()
}

#[doc = include_str!("fontconfig_source.doc/parse_face.md")]
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

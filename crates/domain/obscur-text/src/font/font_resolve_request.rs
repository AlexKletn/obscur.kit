//! A transient request to resolve a font for one itemized run (spec §5.10).

use crate::font::{FontAttributes, FontFallback};
use crate::shaping::ScriptTag;

#[doc = include_str!("../../docs/api/font/font_resolve_request.doc/FontResolveRequest.md")]
#[derive(Debug, Clone, Copy)]
pub struct FontResolveRequest<'a> {
    /// The ordered fallback chain to search for a covering family.
    families: &'a FontFallback,
    /// The weight/slant/width attributes the resolved face should match.
    attributes: &'a FontAttributes,
    /// The run's source text (already itemized to a single script).
    text: &'a str,
    /// The run's resolved script.
    script: ScriptTag,
}

impl<'a> FontResolveRequest<'a> {
    /// Creates a request to resolve a face for `text` (written in `script`) from
    /// `families`, matching `attributes`.
    #[must_use]
    pub const fn new(
        families: &'a FontFallback,
        attributes: &'a FontAttributes,
        text: &'a str,
        script: ScriptTag,
    ) -> Self {
        Self {
            families,
            attributes,
            text,
            script,
        }
    }

    /// Returns the ordered fallback chain to search.
    #[must_use]
    pub const fn families(&self) -> &'a FontFallback {
        self.families
    }

    /// Returns the attributes the resolved face should match.
    #[must_use]
    pub const fn attributes(&self) -> &'a FontAttributes {
        self.attributes
    }

    /// Returns the run's source text.
    #[must_use]
    pub const fn text(&self) -> &'a str {
        self.text
    }

    /// Returns the run's resolved script.
    #[must_use]
    pub const fn script(&self) -> ScriptTag {
        self.script
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontFamily;

    #[test]
    fn accessors_report_construction() {
        let families = FontFallback::new(FontFamily::named("Inter"));
        let attributes = FontAttributes::default();
        let request = FontResolveRequest::new(&families, &attributes, "ab", ScriptTag::LATIN);
        assert_eq!(request.families(), &families);
        assert_eq!(request.attributes(), &attributes);
        assert_eq!(request.text(), "ab");
        assert_eq!(request.script(), ScriptTag::LATIN);
    }
}

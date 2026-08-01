#![doc = include_str!("font_resolve_request.doc/_module.md")]

use crate::font::{FontAttributes, FontFallback};
use crate::shaping::ScriptTag;

#[doc = include_str!("font_resolve_request.doc/FontResolveRequest.md")]
#[derive(Debug, Clone, Copy)]
pub struct FontResolveRequest<'a> {
    #[doc = include_str!("font_resolve_request.doc/families.md")]
    families: &'a FontFallback,
    #[doc = include_str!("font_resolve_request.doc/attributes.md")]
    attributes: &'a FontAttributes,
    #[doc = include_str!("font_resolve_request.doc/text.md")]
    text: &'a str,
    #[doc = include_str!("font_resolve_request.doc/script.md")]
    script: ScriptTag,
}

impl<'a> FontResolveRequest<'a> {
    #[doc = include_str!("font_resolve_request.doc/new.md")]
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

    #[doc = include_str!("font_resolve_request.doc/families_2.md")]
    #[must_use]
    pub const fn families(&self) -> &'a FontFallback {
        self.families
    }

    #[doc = include_str!("font_resolve_request.doc/attributes_2.md")]
    #[must_use]
    pub const fn attributes(&self) -> &'a FontAttributes {
        self.attributes
    }

    #[doc = include_str!("font_resolve_request.doc/text_2.md")]
    #[must_use]
    pub const fn text(&self) -> &'a str {
        self.text
    }

    #[doc = include_str!("font_resolve_request.doc/script_2.md")]
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

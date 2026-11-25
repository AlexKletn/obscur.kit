use crate::shaping::ScriptTag;

use super::{FontFace, FontId, FontResolveRequest, FontResolver, ResolvedFont};

#[doc = include_str!("../../docs/api/font/font_resolver.doc/MockFontResolver.md")]
#[derive(Debug, Default, Clone)]
pub struct MockFontResolver {
    entries: Vec<(ScriptTag, FontId, FontFace)>,
}

impl MockFontResolver {
    /// Creates an empty resolver with no registered scripts; every
    /// [`resolve`](FontResolver::resolve) returns `None` until a script is
    /// registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/font/font_resolver.doc/register.md")]
    pub fn register(&mut self, script: ScriptTag, font: FontId, face: FontFace) {
        if let Some(entry) = self.entries.iter_mut().find(|(s, _, _)| *s == script) {
            entry.1 = font;
            entry.2 = face;
        } else {
            self.entries.push((script, font, face));
        }
    }
}

impl FontResolver for MockFontResolver {
    fn resolve(&self, request: &FontResolveRequest) -> Option<ResolvedFont> {
        self.entries
            .iter()
            .find(|(script, _, _)| *script == request.script())
            .map(|(_, font, face)| ResolvedFont::new(*font, face.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontAttributes, FontFallback, FontFamily, FontMetrics, FontSource};

    fn face(name: &str) -> FontFace {
        FontFace::new(
            FontFamily::named(name),
            FontAttributes::default(),
            FontSource::system(name),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    fn request<'a>(
        families: &'a FontFallback,
        attributes: &'a FontAttributes,
        script: ScriptTag,
    ) -> FontResolveRequest<'a> {
        FontResolveRequest::new(families, attributes, "x", script)
    }

    #[test]
    fn is_object_safe() {
        let resolver: Box<dyn FontResolver> = Box::new(MockFontResolver::new());
        let families = FontFallback::new(FontFamily::named("Inter"));
        let attributes = FontAttributes::default();
        assert!(
            resolver
                .resolve(&request(&families, &attributes, ScriptTag::LATIN))
                .is_none()
        );
    }

    #[test]
    fn resolves_registered_script() {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        resolver.register(ScriptTag::CYRILLIC, FontId::new(1), face("Cyrillic"));

        let families = FontFallback::new(FontFamily::named("Inter"));
        let attributes = FontAttributes::default();

        let latin = resolver
            .resolve(&request(&families, &attributes, ScriptTag::LATIN))
            .expect("latin registered");
        assert_eq!(latin.font(), FontId::new(0));
        assert_eq!(latin.face(), &face("Latin"));

        let cyrillic = resolver
            .resolve(&request(&families, &attributes, ScriptTag::CYRILLIC))
            .expect("cyrillic registered");
        assert_eq!(cyrillic.font(), FontId::new(1));
    }

    #[test]
    fn unregistered_script_is_none() {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("Latin"));
        let families = FontFallback::new(FontFamily::named("Inter"));
        let attributes = FontAttributes::default();
        assert!(
            resolver
                .resolve(&request(&families, &attributes, ScriptTag::ARABIC))
                .is_none()
        );
    }

    #[test]
    fn re_registering_a_script_replaces_it() {
        let mut resolver = MockFontResolver::new();
        resolver.register(ScriptTag::LATIN, FontId::new(0), face("First"));
        resolver.register(ScriptTag::LATIN, FontId::new(5), face("Second"));
        let families = FontFallback::new(FontFamily::named("Inter"));
        let attributes = FontAttributes::default();
        let resolved = resolver
            .resolve(&request(&families, &attributes, ScriptTag::LATIN))
            .expect("latin registered");
        assert_eq!(resolved.font(), FontId::new(5));
        assert_eq!(resolved.face(), &face("Second"));
    }
}

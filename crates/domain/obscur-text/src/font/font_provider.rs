//! The replaceable port for resolving font families to concrete faces (spec
//! §5.18).
//!
//! This module defines [`FontProvider`], the hexagonal port through which the
//! pure text domain turns a requested [`FontFamily`] plus [`FontAttributes`]
//! into a concrete [`FontFace`], and supplies a fallback chain when a family
//! cannot be resolved (spec §5.18). Interface adapters implement it over real
//! font backends — system fonts on native platforms, a bundled fallback set for
//! WASM and isolated environments — while the provider stays fully
//! **replaceable**. The colocated [`MockFontProvider`] is a deterministic
//! reference implementation over an in-memory face registry, so the port is
//! testable without any real font files.

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily, GenericFamily};

/// The replaceable port for resolving fonts (spec §5.18).
///
/// The domain holds a provider as `Box<dyn FontProvider>`, so the trait is
/// deliberately **object-safe** — no generic methods and no `Self`-returning
/// methods. An adapter implements it once per backend; the default provider
/// uses system fonts on native platforms and a bundled fallback set elsewhere,
/// and any adapter can replace it.
pub trait FontProvider {
    /// Resolves `family` and `attributes` to the best available concrete face,
    /// or `None` when the family is unknown to this provider (spec §5.18).
    ///
    /// "Best" means the available face for the family whose attributes are
    /// nearest the request by [`FontAttributes::distance`].
    #[must_use]
    fn resolve(&self, family: &FontFamily, attributes: &FontAttributes) -> Option<FontFace>;

    /// Returns the fallback chain to consult when `family` cannot be resolved
    /// (spec §5.18 fallback resolution, §5.10 font fallback).
    #[must_use]
    fn fallback(&self, family: &FontFamily) -> FontFallback;

    /// Returns whether this provider can supply any face for `family`
    /// (spec §5.18).
    #[must_use]
    fn is_available(&self, family: &FontFamily) -> bool;
}

/// A deterministic reference [`FontProvider`] backed by an in-memory registry of
/// faces (spec §5.18).
///
/// `MockFontProvider` holds the faces registered via [`register`](Self::register)
/// and resolves against them with no real font files, which makes the port
/// exercisable in tests. Its [`resolve`](FontProvider::resolve) picks the
/// registered face whose family matches by [`FontFamily::name`] with the
/// smallest [`FontAttributes::distance`]; ties are broken deterministically in
/// favour of the **first** registered face. Its
/// [`fallback`](FontProvider::fallback) returns the requested family followed by
/// the generic `sans-serif`.
#[derive(Debug, Default, Clone)]
pub struct MockFontProvider {
    faces: Vec<(FontFamily, FontFace)>,
}

impl MockFontProvider {
    /// Creates an empty provider with no registered faces.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `face`, keyed by its own family, as an available face.
    ///
    /// Registration order is significant: on an exact tie in resolution
    /// distance, the earliest-registered matching face wins.
    pub fn register(&mut self, face: FontFace) {
        self.faces.push((face.family().clone(), face));
    }
}

impl FontProvider for MockFontProvider {
    fn resolve(&self, family: &FontFamily, attributes: &FontAttributes) -> Option<FontFace> {
        let mut best: Option<(&FontFace, u32)> = None;
        for (registered_family, face) in &self.faces {
            if registered_family.name() != family.name() {
                continue;
            }
            let distance = attributes.distance(face.attributes());
            // Strict `<` keeps the first-registered face on an exact tie.
            if best.is_none_or(|(_, best_distance)| distance < best_distance) {
                best = Some((face, distance));
            }
        }
        best.map(|(face, _)| face.clone())
    }

    fn fallback(&self, family: &FontFamily) -> FontFallback {
        FontFallback::new(family.clone()).with(FontFamily::generic(GenericFamily::SansSerif))
    }

    fn is_available(&self, family: &FontFamily) -> bool {
        self.faces
            .iter()
            .any(|(registered_family, _)| registered_family.name() == family.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontMetrics, FontSource};
    use crate::span::FontWeight;

    fn face(family: &FontFamily, weight: FontWeight) -> FontFace {
        FontFace::new(
            family.clone(),
            FontAttributes::new().with_weight(weight),
            FontSource::system("stub"),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn FontProvider>`.
    #[test]
    fn font_provider_is_object_safe() {
        let provider: Box<dyn FontProvider> = Box::new(MockFontProvider::new());
        assert!(!provider.is_available(&FontFamily::named("Nothing")));
    }

    #[test]
    fn resolve_picks_nearest_weight() {
        let inter = FontFamily::named("Inter");
        let mut provider = MockFontProvider::new();
        provider.register(face(&inter, FontWeight::new(300)));
        provider.register(face(&inter, FontWeight::new(700)));

        // A request for 600 is nearer 700 than 300.
        let request = FontAttributes::new().with_weight(FontWeight::new(600));
        let resolved = provider.resolve(&inter, &request).expect("resolves");
        assert_eq!(resolved.attributes().weight, FontWeight::new(700));

        // A request for 350 is nearer 300.
        let request = FontAttributes::new().with_weight(FontWeight::new(350));
        let resolved = provider.resolve(&inter, &request).expect("resolves");
        assert_eq!(resolved.attributes().weight, FontWeight::new(300));
    }

    #[test]
    fn resolve_tie_break_first_registered_wins() {
        let inter = FontFamily::named("Inter");
        let mut provider = MockFontProvider::new();
        // Two faces equidistant from the request (400 is 100 from each).
        provider.register(face(&inter, FontWeight::new(300)));
        provider.register(face(&inter, FontWeight::new(500)));
        let request = FontAttributes::new().with_weight(FontWeight::new(400));
        let resolved = provider.resolve(&inter, &request).expect("resolves");
        assert_eq!(resolved.attributes().weight, FontWeight::new(300));
    }

    #[test]
    fn resolve_unknown_family_is_none() {
        let provider = MockFontProvider::new();
        assert!(provider
            .resolve(&FontFamily::named("Ghost"), &FontAttributes::new())
            .is_none());
    }

    #[test]
    fn is_available_matches_by_name() {
        let inter = FontFamily::named("Inter");
        let mut provider = MockFontProvider::new();
        provider.register(face(&inter, FontWeight::NORMAL));
        assert!(provider.is_available(&inter));
        assert!(!provider.is_available(&FontFamily::named("Roboto")));
    }

    #[test]
    fn fallback_chain_ends_in_sans_serif() {
        let provider = MockFontProvider::new();
        let chain = provider.fallback(&FontFamily::named("Inter"));
        assert_eq!(chain.primary(), Some(&FontFamily::named("Inter")));
        assert_eq!(
            chain.families().last(),
            Some(&FontFamily::generic(GenericFamily::SansSerif))
        );
    }
}

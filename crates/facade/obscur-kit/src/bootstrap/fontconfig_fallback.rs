//! [`FontconfigFallback`]: the host's per-character font lookup, wired to the
//! text stack (spec §5.10 font fallback, §5.18).

use std::sync::{Arc, OnceLock};

use crate::font_fontconfig::source::{FontconfigSource, SystemFace};
use crate::text::font::{FallbackFace, FontAttributes, FontFamily, SystemFontFallback};

#[doc = include_str!("../../docs/api/bootstrap/fontconfig_fallback.doc/FontconfigFallback.md")]
#[derive(Debug, Clone)]
pub struct FontconfigFallback {
    source: FontconfigSource,
    catalog: Arc<OnceLock<Vec<SystemFace>>>,
}

impl FontconfigFallback {
    /// Creates a fallback over the conventional `fc-match`.
    #[must_use]
    pub fn new() -> Self {
        let source = FontconfigSource::new();
        let catalog = Arc::new(OnceLock::new());
        if std::env::var_os("OBSCUR_FONTCONFIG_SYNC_LOOKUP").as_deref() != Some("1".as_ref()) {
            let source = source.clone();
            let catalog = Arc::clone(&catalog);
            let _ = std::thread::Builder::new()
                .name("obscur-fontconfig".into())
                .spawn(move || {
                    if let Some(faces) = source.list_faces() {
                        let _ = catalog.set(faces);
                    }
                });
        }
        Self { source, catalog }
    }
}

impl Default for FontconfigFallback {
    fn default() -> Self {
        Self::new()
    }
}

fn load_named_face(face: &SystemFace) -> Option<FallbackFace> {
    let bytes = std::fs::read(face.path()).ok()?;
    Some(
        FallbackFace::new(FontFamily::named(face.family()), Arc::from(bytes))
            .with_attributes(*face.attributes()),
    )
}

impl SystemFontFallback for FontconfigFallback {
    fn face_for_char(&self, character: char) -> Option<FallbackFace> {
        let face = self.source.match_char(character)?;
        let bytes = std::fs::read(face.path()).ok()?;
        Some(FallbackFace::new(
            FontFamily::named(face.family()),
            Arc::from(bytes),
        ))
    }

    fn face_for_family(&self, name: &str, attributes: &FontAttributes) -> Option<FallbackFace> {
        self.catalog
            .get()
            .and_then(|faces| FontconfigSource::closest_family(faces, name, attributes))
            .and_then(load_named_face)
            .or_else(|| {
                self.source
                    .find_family(name, attributes)
                    .as_ref()
                    .and_then(load_named_face)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Latin is covered by the faces already in the chain, but this asks the host
    /// directly: whatever it answers must at least be a *loadable* face, since a
    /// path that cannot be read is dropped rather than returned empty.
    ///
    /// Skipped where fontconfig is absent — there is nothing to ask.
    #[test]
    fn a_host_answer_is_a_real_readable_face() {
        let fallback = FontconfigFallback::new();
        if !FontconfigSource::new().is_available() {
            return;
        }
        if let Some(face) = fallback.face_for_char('A') {
            assert!(
                !face.bytes().is_empty(),
                "a returned face carries its bytes"
            );
            assert!(!face.family().name().is_empty());
        }
    }

    /// A name no machine has must come back as [`None`] rather than as the
    /// substitute `fc-match` would happily hand over — otherwise a font that is
    /// not installed is indistinguishable from one that is.
    #[test]
    fn an_uninstalled_family_is_not_answered_with_a_substitute() {
        let fallback = FontconfigFallback::new();
        if !FontconfigSource::new().is_available() {
            return;
        }
        assert!(
            fallback
                .face_for_family("ThisFamilyDoesNotExist12345", &FontAttributes::new())
                .is_none()
        );
    }
}

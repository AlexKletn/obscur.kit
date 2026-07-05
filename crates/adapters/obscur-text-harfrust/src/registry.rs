//! The [`FontRegistry`] adapter implementing [`FontResolver`] with real,
//! `cmap`-coverage-driven font fallback (spec §5.10).

use std::sync::Arc;

use obscur_text::font::{
    FontAttributes, FontFace, FontFamily, FontId, FontMetrics, FontResolveRequest, FontResolver,
    FontSource, ResolvedFont,
};
use read_fonts::{FontRef as ReadFontRef, TableProvider};

/// A real [`FontResolver`] backed by a set of registered fonts and their true
/// `cmap` coverage (spec §5.10 font fallback).
///
/// Fonts are added with [`register`](Self::register), which parses each face
/// once (via [`read_fonts`]) to read its `units_per_em` and vertical metrics and
/// hands back a [`FontId`] assigned in registration order (`0, 1, 2, …`). The id
/// is deterministic and stable, so a later renderer can look the same face back
/// up by id.
///
/// [`resolve`](FontResolver::resolve) walks the request's fallback chain in
/// order (primary first). For each family it consults every registered font
/// whose family name matches, and returns the **first** one whose `cmap`
/// actually covers the run — i.e. every non-whitespace character maps to a
/// non-zero glyph id. When nothing in the chain covers the run it returns
/// `None`, leaving the orchestrator to fall back to its default face.
///
/// Coverage is checked by re-parsing the font's `cmap` per `resolve` call
/// (cheap: resolution happens once per itemized run, not per glyph, and an
/// upstream cache elides repeats), so the registry holds only the raw bytes and
/// pre-read metrics — no borrowed, self-referential font handle.
#[derive(Debug, Clone, Default)]
pub struct FontRegistry {
    fonts: Vec<RegisteredFont>,
}

/// One registered font: its id, requested family/attributes, raw bytes, and the
/// vertical metrics read once at registration time.
#[derive(Debug, Clone)]
struct RegisteredFont {
    id: FontId,
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
    units_per_em: u16,
    ascent: f32,
    descent: f32,
    line_gap: f32,
}

impl FontRegistry {
    /// Creates an empty registry; every [`resolve`](FontResolver::resolve)
    /// returns `None` until a font is registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a font (`bytes`) under `family`/`attributes` and returns the
    /// [`FontId`] assigned to it — the next index in registration order (`0` for
    /// the first font, `1` for the second, …).
    ///
    /// The bytes are parsed once to read the face's `units_per_em` and `hhea`
    /// vertical metrics for the resolved [`FontFace`]. Bytes that do not parse as
    /// a font are still registered (keeping id assignment deterministic) with
    /// idealized default metrics; such a font simply never covers any run.
    pub fn register(
        &mut self,
        family: FontFamily,
        attributes: FontAttributes,
        bytes: impl Into<Arc<[u8]>>,
    ) -> FontId {
        let bytes = bytes.into();
        // Ids are handed out densely in registration order, so the index is the
        // current length.
        let id = FontId::new(u32::try_from(self.fonts.len()).unwrap_or(u32::MAX));
        let (units_per_em, ascent, descent, line_gap) = read_metrics(&bytes);
        self.fonts.push(RegisteredFont {
            id,
            family,
            attributes,
            bytes,
            units_per_em,
            ascent,
            descent,
            line_gap,
        });
        id
    }

    /// Returns the number of registered fonts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// Returns whether no fonts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl FontResolver for FontRegistry {
    fn resolve(&self, request: &FontResolveRequest) -> Option<ResolvedFont> {
        let text = request.text();
        let requested = request.attributes();
        for family in request.families() {
            // Among the same-family faces that cover the run, pick the one whose
            // attributes are nearest the request (smallest `FontAttributes::distance`).
            // `min_by_key` keeps the **first** face on a tie, so equal-distance
            // faces resolve in registration order — a stable, deterministic choice.
            let best = self
                .fonts
                .iter()
                .filter(|font| font.family.name() == family.name())
                .filter(|font| font.covers(text))
                .min_by_key(|font| font.attributes.distance(requested));
            if let Some(font) = best {
                let face = FontFace::new(
                    font.family.clone(),
                    font.attributes,
                    FontSource::memory(Arc::clone(&font.bytes)),
                    FontMetrics::new(font.units_per_em, font.ascent, font.descent, font.line_gap),
                );
                return Some(ResolvedFont::new(font.id, face));
            }
        }
        None
    }
}

impl RegisteredFont {
    /// Returns whether this font's `cmap` covers every meaningful character of
    /// `text` — i.e. each maps to a non-zero (non-`.notdef`) glyph id.
    ///
    /// ASCII whitespace and control characters are treated as always covered, so
    /// a run like `"ab cd"` is not rejected for its space. Bytes that fail to
    /// parse (or lack a `cmap`) cover nothing.
    fn covers(&self, text: &str) -> bool {
        let Ok(font) = ReadFontRef::new(&self.bytes) else {
            return false;
        };
        let Ok(cmap) = font.cmap() else {
            return false;
        };
        text.chars()
            .filter(|ch| !is_ignorable(*ch))
            .all(|ch| cmap.map_codepoint(ch).is_some_and(|gid| gid.to_u32() != 0))
    }
}

/// Reads `(units_per_em, ascent, descent, line_gap)` in font design units from
/// the `head`/`hhea` tables, falling back to idealized defaults (1000 upem,
/// 800 / −200 / 0) when the bytes are not a loadable font or the tables are
/// missing. `descent` keeps the font's signed (negative-below-baseline)
/// convention, matching [`FontMetrics`].
fn read_metrics(bytes: &[u8]) -> (u16, f32, f32, f32) {
    let read = || -> Option<(u16, f32, f32, f32)> {
        let font = ReadFontRef::new(bytes).ok()?;
        let units_per_em = font.head().ok()?.units_per_em();
        if units_per_em == 0 {
            return None;
        }
        let hhea = font.hhea().ok()?;
        Some((
            units_per_em,
            f32::from(hhea.ascender().to_i16()),
            f32::from(hhea.descender().to_i16()),
            f32::from(hhea.line_gap().to_i16()),
        ))
    };
    read().unwrap_or((1000, 800.0, -200.0, 0.0))
}

/// Returns whether `ch` is ignored for coverage: ASCII whitespace or an ASCII
/// control character, which need no glyph.
const fn is_ignorable(ch: char) -> bool {
    ch.is_ascii_whitespace() || ch.is_ascii_control()
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_text::font::FontFallback;
    use obscur_text::shaping::ScriptTag;

    /// Builds a tiny but valid TrueType font carrying a format-6 Unicode `cmap`
    /// that maps `first_code + i` to `gids[i]`, plus real `head`/`hhea`/`hmtx`
    /// tables. `units_per_em` is 1000 and `hhea` reports ascender 800 /
    /// descender −200 / lineGap 0. A gid of `0` means the code is present in the
    /// range but resolves to `.notdef` (uncovered). This is a genuine sfnt
    /// `read-fonts` can parse, so coverage tests stay fully hermetic.
    fn build_font(first_code: u16, gids: &[u16]) -> Vec<u8> {
        let num_glyphs = gids.iter().copied().max().unwrap_or(0).max(2) + 1;

        fn head() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&0u32.to_be_bytes()); // fontRevision
            t.extend_from_slice(&0u32.to_be_bytes()); // checksumAdjustment
            t.extend_from_slice(&0x5F0F_3CF5u32.to_be_bytes()); // magicNumber
            t.extend_from_slice(&0u16.to_be_bytes()); // flags
            t.extend_from_slice(&1000u16.to_be_bytes()); // unitsPerEm
            t.extend_from_slice(&0u64.to_be_bytes()); // created
            t.extend_from_slice(&0u64.to_be_bytes()); // modified
            t.extend_from_slice(&0i16.to_be_bytes()); // xMin
            t.extend_from_slice(&0i16.to_be_bytes()); // yMin
            t.extend_from_slice(&700i16.to_be_bytes()); // xMax
            t.extend_from_slice(&700i16.to_be_bytes()); // yMax
            t.extend_from_slice(&0u16.to_be_bytes()); // macStyle
            t.extend_from_slice(&8u16.to_be_bytes()); // lowestRecPPEM
            t.extend_from_slice(&0i16.to_be_bytes()); // fontDirectionHint
            t.extend_from_slice(&0i16.to_be_bytes()); // indexToLocFormat (short)
            t.extend_from_slice(&0i16.to_be_bytes()); // glyphDataFormat
            t
        }

        fn hhea(num_glyphs: u16) -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&800i16.to_be_bytes()); // ascender
            t.extend_from_slice(&(-200i16).to_be_bytes()); // descender
            t.extend_from_slice(&0i16.to_be_bytes()); // lineGap
            t.extend_from_slice(&[0u8; 24]); // advanceWidthMax .. metricDataFormat
            t.extend_from_slice(&num_glyphs.to_be_bytes()); // numberOfHMetrics
            t
        }

        fn maxp(num_glyphs: u16) -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0000_5000u32.to_be_bytes()); // version 0.5
            t.extend_from_slice(&num_glyphs.to_be_bytes()); // numGlyphs
            t
        }

        fn hmtx(num_glyphs: u16) -> Vec<u8> {
            let mut t = Vec::new();
            // longHorMetric { advanceWidth: u16, lsb: i16 } per glyph.
            for _ in 0..num_glyphs {
                t.extend_from_slice(&500u16.to_be_bytes());
                t.extend_from_slice(&0i16.to_be_bytes());
            }
            t
        }

        fn loca(num_glyphs: u16) -> Vec<u8> {
            // Short loca: numGlyphs + 1 zero entries (all empty glyphs).
            vec![0u8; usize::from(num_glyphs + 1) * 2]
        }

        let cmap = {
            let count = u16::try_from(gids.len()).expect("cmap entry count fits u16");
            let mut sub = Vec::new();
            sub.extend_from_slice(&6u16.to_be_bytes()); // format
            sub.extend_from_slice(&(10 + count * 2).to_be_bytes()); // length
            sub.extend_from_slice(&0u16.to_be_bytes()); // language
            sub.extend_from_slice(&first_code.to_be_bytes()); // firstCode
            sub.extend_from_slice(&count.to_be_bytes()); // entryCount
            for gid in gids {
                sub.extend_from_slice(&gid.to_be_bytes());
            }

            let mut t = Vec::new();
            t.extend_from_slice(&0u16.to_be_bytes()); // version
            t.extend_from_slice(&1u16.to_be_bytes()); // numTables
            t.extend_from_slice(&3u16.to_be_bytes()); // platformID (Windows)
            t.extend_from_slice(&1u16.to_be_bytes()); // encodingID (Unicode BMP)
            t.extend_from_slice(&12u32.to_be_bytes()); // subtable offset
            t.extend_from_slice(&sub);
            t
        };

        // Directory records must be tag-sorted.
        let tables: [(&[u8; 4], Vec<u8>); 7] = [
            (b"cmap", cmap),
            (b"glyf", Vec::new()),
            (b"head", head()),
            (b"hhea", hhea(num_glyphs)),
            (b"hmtx", hmtx(num_glyphs)),
            (b"loca", loca(num_glyphs)),
            (b"maxp", maxp(num_glyphs)),
        ];

        let num_tables = tables.len() as u16;
        let mut offset = 12 + tables.len() * 16;
        let mut records = Vec::new();
        let mut body = Vec::new();
        for (tag, data) in &tables {
            records.extend_from_slice(*tag);
            records.extend_from_slice(&0u32.to_be_bytes()); // checksum (unchecked)
            records.extend_from_slice(&(offset as u32).to_be_bytes());
            records.extend_from_slice(&(data.len() as u32).to_be_bytes());
            body.extend_from_slice(data);
            let pad = (4 - data.len() % 4) % 4;
            body.extend(std::iter::repeat_n(0u8, pad));
            offset += data.len() + pad;
        }

        let mut font = Vec::new();
        font.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // sfnt version
        font.extend_from_slice(&num_tables.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes()); // searchRange
        font.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
        font.extend_from_slice(&0u16.to_be_bytes()); // rangeShift
        font.extend_from_slice(&records);
        font.extend_from_slice(&body);
        font
    }

    /// Font covering 'A' (gid 1) and 'V' (gid 2); everything between maps to
    /// `.notdef`.
    fn alpha_font() -> Vec<u8> {
        // 0x41..=0x56: 'A' -> 1, 'V' -> 2, rest -> 0 (uncovered).
        let mut gids = vec![0u16; 0x16];
        gids[0] = 1; // 'A' == 0x41
        gids[0x56 - 0x41] = 2; // 'V' == 0x56
        build_font(0x41, &gids)
    }

    /// Font covering only Cyrillic 'Я' (U+042F -> gid 1).
    fn beta_font() -> Vec<u8> {
        build_font(0x042F, &[1])
    }

    fn request<'a>(
        families: &'a FontFallback,
        attributes: &'a FontAttributes,
        text: &'a str,
    ) -> FontResolveRequest<'a> {
        FontResolveRequest::new(families, attributes, text, ScriptTag::LATIN)
    }

    fn alpha_beta_chain() -> FontFallback {
        FontFallback::new(FontFamily::named("Alpha")).with(FontFamily::named("Beta"))
    }

    fn registry() -> FontRegistry {
        let mut registry = FontRegistry::new();
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default(),
            alpha_font(),
        );
        registry.register(
            FontFamily::named("Beta"),
            FontAttributes::default(),
            beta_font(),
        );
        registry
    }

    #[test]
    fn registration_assigns_sequential_ids() {
        let mut registry = FontRegistry::new();
        let a = registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default(),
            alpha_font(),
        );
        let b = registry.register(
            FontFamily::named("Beta"),
            FontAttributes::default(),
            beta_font(),
        );
        assert_eq!(a, FontId::new(0));
        assert_eq!(b, FontId::new(1));
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    #[test]
    fn primary_family_covering_run_resolves_to_its_font() {
        let registry = registry();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        let resolved = registry
            .resolve(&request(&chain, &attributes, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(0));
        assert_eq!(resolved.face().family(), &FontFamily::named("Alpha"));
    }

    #[test]
    fn falls_through_non_covering_primary_to_covering_fallback() {
        let registry = registry();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        // 'Я' is not in Alpha's cmap range, so resolution must skip Alpha and
        // pick Beta from the chain.
        let resolved = registry
            .resolve(&request(&chain, &attributes, "Я"))
            .expect("Beta covers 'Я'");
        assert_eq!(resolved.font(), FontId::new(1));
        assert_eq!(resolved.face().family(), &FontFamily::named("Beta"));
    }

    #[test]
    fn run_no_font_covers_resolves_to_none() {
        let registry = registry();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        // 'Ω' (U+03A9) is covered by neither font.
        assert!(registry.resolve(&request(&chain, &attributes, "Ω")).is_none());
    }

    #[test]
    fn resolved_face_carries_memory_bytes_and_real_metrics() {
        let registry = registry();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        let resolved = registry
            .resolve(&request(&chain, &attributes, "AV"))
            .expect("Alpha covers 'A' and 'V'");

        // The face carries the exact registered bytes as an in-memory source.
        match resolved.face().source() {
            FontSource::Memory(bytes) => assert_eq!(&bytes[..], &alpha_font()[..]),
            other => panic!("expected Memory source, got {other:?}"),
        }
        // Metrics are read from the font's head/hhea, not placeholders.
        let metrics = resolved.face().metrics();
        assert_eq!(metrics.units_per_em, 1000);
        assert_eq!(metrics.ascent, 800.0);
        assert_eq!(metrics.descent, -200.0);
        assert_eq!(metrics.line_gap, 0.0);
    }

    #[test]
    fn whitespace_in_run_does_not_reject_coverage() {
        let registry = registry();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        // The space and tab are ignorable; 'A' and 'V' are both in Alpha.
        let resolved = registry
            .resolve(&request(&chain, &attributes, "A\tV A"))
            .expect("whitespace is always covered");
        assert_eq!(resolved.font(), FontId::new(0));
    }

    #[test]
    fn empty_registry_resolves_to_none() {
        let registry = FontRegistry::new();
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        assert!(registry.resolve(&request(&chain, &attributes, "A")).is_none());
    }

    #[test]
    fn family_not_in_chain_is_not_resolved() {
        let registry = registry();
        // A chain naming only "Alpha" cannot reach Beta even for 'Я'.
        let chain = FontFallback::new(FontFamily::named("Alpha"));
        let attributes = FontAttributes::default();
        assert!(registry.resolve(&request(&chain, &attributes, "Я")).is_none());
    }

    #[test]
    fn nearest_weight_of_same_family_is_selected() {
        use obscur_text::span::FontWeight;

        // Two cuts of one family, both covering 'A': a regular (400) and a bold
        // (700). Registration order is regular first (id 0), bold second (id 1).
        let mut registry = FontRegistry::new();
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default().with_weight(FontWeight::NORMAL),
            alpha_font(),
        );
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default().with_weight(FontWeight::BOLD),
            alpha_font(),
        );
        let chain = FontFallback::new(FontFamily::named("Alpha"));

        // A bold request resolves to the bold cut (id 1), not the first-registered
        // regular, because its attributes are nearer.
        let bold = FontAttributes::default().with_weight(FontWeight::BOLD);
        let resolved = registry
            .resolve(&request(&chain, &bold, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(1));

        // A light request resolves to the regular cut (id 0), the nearer weight.
        let light = FontAttributes::default().with_weight(FontWeight::LIGHT);
        let resolved = registry
            .resolve(&request(&chain, &light, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(0));
    }

    #[test]
    fn is_object_safe() {
        let resolver: Box<dyn FontResolver> = Box::new(registry());
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        assert!(resolver.resolve(&request(&chain, &attributes, "A")).is_some());
    }
}

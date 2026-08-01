#![doc = include_str!("registry.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use obscur_text::font::{
    FontAttributes, FontFace, FontFamily, FontId, FontMetrics, FontResolveRequest, FontResolver,
    FontSource, ResolvedFont, SystemFontFallback,
};
use obscur_text::span::FontSlant;
use read_fonts::{FontRef as ReadFontRef, TableProvider};

use crate::face_metrics::FaceMetrics;

#[doc = include_str!("registry.doc/FontRegistry.md")]
#[derive(Clone, Default)]
pub struct FontRegistry {
    fonts: Vec<RegisteredFont>,
    #[doc = include_str!("registry.doc/discovered.md")]
    discovered: Rc<RefCell<Vec<RegisteredFont>>>,
    #[doc = include_str!("registry.doc/undelivered.md")]
    undelivered: UndeliveredFaces,
    #[doc = include_str!("registry.doc/fallback.md")]
    fallback: Option<Arc<dyn SystemFontFallback>>,
}

impl std::fmt::Debug for FontRegistry {
    #[doc = include_str!("registry.doc/fmt.md")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontRegistry")
            .field("fonts", &self.fonts)
            .field("discovered", &self.discovered)
            .field("undelivered", &self.undelivered.borrow().len())
            .field(
                "fallback",
                &self.fallback.as_ref().map(|_| "<SystemFontFallback>"),
            )
            .finish()
    }
}

#[doc = include_str!("registry.doc/UndeliveredFaces.md")]
type UndeliveredFaces = Rc<RefCell<Vec<(FontId, Arc<[u8]>)>>>;

#[doc = include_str!("registry.doc/RegisteredFont.md")]
#[derive(Debug, Clone)]
struct RegisteredFont {
    id: FontId,
    family: FontFamily,
    attributes: FontAttributes,
    bytes: Arc<[u8]>,
    metrics: Option<FontMetrics>,
}

impl FontRegistry {
    #[doc = include_str!("registry.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("registry.doc/register.md")]
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
        let metrics = read_metrics(&bytes);
        self.fonts.push(RegisteredFont {
            id,
            family,
            attributes,
            bytes,
            metrics,
        });
        id
    }

    #[doc = include_str!("registry.doc/with_system_fallback.md")]
    #[must_use]
    pub fn with_system_fallback(mut self, fallback: Arc<dyn SystemFontFallback>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    #[doc = include_str!("registry.doc/take_undelivered_faces.md")]
    #[must_use]
    pub fn take_undelivered_faces(&self) -> Vec<(FontId, Arc<[u8]>)> {
        std::mem::take(&mut self.undelivered.borrow_mut())
    }

    #[doc = include_str!("registry.doc/discover.md")]
    fn discover(&self, text: &str) -> Option<usize> {
        let fallback = self.fallback.as_ref()?;
        // The first character no face — registered or already discovered — can
        // draw. Anything else is already answerable and asking about it would
        // pull in a face nobody needs.
        let uncovered = text.chars().find(|ch| {
            !is_ignorable(*ch)
                && !self.fonts.iter().any(|font| font.covers_char(*ch))
                && !self
                    .discovered
                    .borrow()
                    .iter()
                    .any(|font| font.covers_char(*ch))
        })?;
        let face = fallback.face_for_char(uncovered)?;
        // Trust but verify: a host that answers with a face lacking the very
        // character asked about would otherwise be registered forever as the
        // wrong answer for it.
        let bytes = Arc::clone(face.bytes());
        let candidate = RegisteredFont {
            id: self.next_id(),
            family: face.family().clone(),
            attributes: FontAttributes::default(),
            metrics: read_metrics(&bytes),
            bytes,
        };
        if !candidate.covers_char(uncovered) {
            return None;
        }
        self.undelivered
            .borrow_mut()
            .push((candidate.id, Arc::clone(&candidate.bytes)));
        let mut discovered = self.discovered.borrow_mut();
        discovered.push(candidate);
        Some(discovered.len() - 1)
    }

    #[doc = include_str!("registry.doc/next_id.md")]
    fn next_id(&self) -> FontId {
        let used = self.fonts.len() + self.discovered.borrow().len();
        FontId::new(u32::try_from(used).unwrap_or(u32::MAX))
    }

    #[doc = include_str!("registry.doc/face_metrics.md")]
    #[must_use]
    pub fn face_metrics(&self) -> FaceMetrics {
        FaceMetrics::new(
            self.fonts
                .iter()
                .map(|font| font.metrics)
                .collect::<Vec<_>>(),
        )
    }

    #[doc = include_str!("registry.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    #[doc = include_str!("registry.doc/is_empty.md")]
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
                    font.metrics.unwrap_or(IDEALIZED_METRICS),
                );
                // Detect synthesis: the request wants a slant/weight the matched
                // face — chosen as the nearest available cut — does not actually
                // carry, so the renderer must fake it (spec §5.18).
                let (oblique, bold) = synthesis(requested, &font.attributes);
                return Some(ResolvedFont::new(font.id, face).with_synthesis(oblique, bold));
            }
        }
        // Nothing the application named can draw this run. Before giving up —
        // which means drawing *nothing*, since an unresolved run paints no
        // glyphs — consult the faces already discovered for earlier runs, then
        // ask the host for one covering the first character none of them has.
        let already = self
            .discovered
            .borrow()
            .iter()
            .position(|font| font.covers(text));
        let index = match already {
            Some(index) => index,
            None => self.discover(text)?,
        };
        let discovered = self.discovered.borrow();
        let font = discovered.get(index)?;
        let face = FontFace::new(
            font.family.clone(),
            font.attributes,
            FontSource::memory(Arc::clone(&font.bytes)),
            font.metrics.unwrap_or(IDEALIZED_METRICS),
        );
        let (oblique, bold) = synthesis(requested, &font.attributes);
        Some(ResolvedFont::new(font.id, face).with_synthesis(oblique, bold))
    }
}

impl RegisteredFont {
    #[doc = include_str!("registry.doc/covers_char.md")]
    fn covers_char(&self, character: char) -> bool {
        let Ok(font) = ReadFontRef::new(&self.bytes) else {
            return false;
        };
        let Ok(cmap) = font.cmap() else {
            return false;
        };
        cmap.map_codepoint(character)
            .is_some_and(|gid| gid.to_u32() != 0)
    }

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

#[doc = include_str!("registry.doc/IDEALIZED_METRICS.md")]
const IDEALIZED_METRICS: FontMetrics = FontMetrics::new(1000, 800.0, -200.0, 0.0);

#[doc = include_str!("registry.doc/read_metrics.md")]
fn read_metrics(bytes: &[u8]) -> Option<FontMetrics> {
    let font = ReadFontRef::new(bytes).ok()?;
    let units_per_em = font.head().ok()?.units_per_em();
    if units_per_em == 0 {
        return None;
    }
    let hhea = font.hhea().ok()?;
    Some(FontMetrics::new(
        units_per_em,
        f32::from(hhea.ascender().to_i16()),
        f32::from(hhea.descender().to_i16()),
        f32::from(hhea.line_gap().to_i16()),
    ))
}

#[doc = include_str!("registry.doc/BOLD_THRESHOLD.md")]
const BOLD_THRESHOLD: u16 = 600;

#[doc = include_str!("registry.doc/synthesis.md")]
fn synthesis(requested: &FontAttributes, matched: &FontAttributes) -> (bool, bool) {
    let oblique = requested.slant != FontSlant::Normal && matched.slant == FontSlant::Normal;
    let bold =
        requested.weight.value() >= BOLD_THRESHOLD && matched.weight.value() < BOLD_THRESHOLD;
    (oblique, bold)
}

#[doc = include_str!("registry.doc/is_ignorable.md")]
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
        assert!(
            registry
                .resolve(&request(&chain, &attributes, "Ω"))
                .is_none()
        );
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
        assert!(
            registry
                .resolve(&request(&chain, &attributes, "A"))
                .is_none()
        );
    }

    #[test]
    fn family_not_in_chain_is_not_resolved() {
        let registry = registry();
        // A chain naming only "Alpha" cannot reach Beta even for 'Я'.
        let chain = FontFallback::new(FontFamily::named("Alpha"));
        let attributes = FontAttributes::default();
        assert!(
            registry
                .resolve(&request(&chain, &attributes, "Я"))
                .is_none()
        );
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
    fn italic_request_against_upright_face_flags_synthetic_oblique() {
        use obscur_text::span::FontSlant;

        // Only an upright cut of "Alpha" is registered.
        let mut registry = FontRegistry::new();
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default(),
            alpha_font(),
        );
        let chain = FontFallback::new(FontFamily::named("Alpha"));

        // An italic request has no true italic cut → resolve to the upright face
        // but flag it for synthetic oblique.
        let italic = FontAttributes::default().with_slant(FontSlant::Italic);
        let resolved = registry
            .resolve(&request(&chain, &italic, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(0));
        assert!(resolved.synth_oblique(), "upright face must be sheared");
        assert!(!resolved.synth_bold());

        // An upright request needs no synthesis.
        let upright = FontAttributes::default();
        let resolved = registry
            .resolve(&request(&chain, &upright, "A"))
            .expect("Alpha covers 'A'");
        assert!(!resolved.synth_oblique());
    }

    #[test]
    fn italic_request_with_real_italic_face_is_not_synthesized() {
        use obscur_text::span::FontSlant;

        // Both an upright and a true italic cut of "Alpha" are registered.
        let mut registry = FontRegistry::new();
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default(),
            alpha_font(),
        );
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default().with_slant(FontSlant::Italic),
            alpha_font(),
        );
        let chain = FontFallback::new(FontFamily::named("Alpha"));

        // The italic request resolves to the real italic cut (id 1), no synthesis.
        let italic = FontAttributes::default().with_slant(FontSlant::Italic);
        let resolved = registry
            .resolve(&request(&chain, &italic, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(1));
        assert!(
            !resolved.synth_oblique(),
            "a real italic face must not be faux-sheared"
        );
    }

    #[test]
    fn bold_request_against_regular_face_flags_synthetic_bold() {
        use obscur_text::span::FontWeight;

        // Only a regular (400) cut is registered.
        let mut registry = FontRegistry::new();
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default().with_weight(FontWeight::NORMAL),
            alpha_font(),
        );
        let chain = FontFallback::new(FontFamily::named("Alpha"));

        // A bold (700) request has no true bold cut → flag synthetic bold.
        let bold = FontAttributes::default().with_weight(FontWeight::BOLD);
        let resolved = registry
            .resolve(&request(&chain, &bold, "A"))
            .expect("Alpha covers 'A'");
        assert!(resolved.synth_bold(), "regular face must be thickened");
        assert!(!resolved.synth_oblique());

        // With a real bold cut registered, the bold request is not synthesized.
        registry.register(
            FontFamily::named("Alpha"),
            FontAttributes::default().with_weight(FontWeight::BOLD),
            alpha_font(),
        );
        let resolved = registry
            .resolve(&request(&chain, &bold, "A"))
            .expect("Alpha covers 'A'");
        assert_eq!(resolved.font(), FontId::new(1));
        assert!(
            !resolved.synth_bold(),
            "a real bold face must not be faux-bolded"
        );
    }

    #[test]
    fn face_metrics_snapshot_is_keyed_by_the_issued_ids() {
        let registry = registry();
        let faces = registry.face_metrics();
        assert_eq!(faces.len(), registry.len());
        // Both fonts carry the same real head/hhea (1000 upem, 800 / −200 / 0),
        // read once at registration and reachable by the id `register` returned.
        for id in [FontId::new(0), FontId::new(1)] {
            let metrics = faces.get(id).expect("a loadable face has real metrics");
            assert_eq!(metrics.units_per_em, 1000);
            assert_eq!(metrics.ascent, 800.0);
            assert_eq!(metrics.descent, -200.0);
        }
        // An id the registry never issued names no face.
        assert!(faces.get(FontId::new(2)).is_none());
    }

    #[test]
    fn an_unloadable_faces_metrics_are_reported_as_absent_not_faked() {
        let mut registry = FontRegistry::new();
        let id = registry.register(
            FontFamily::named("Broken"),
            FontAttributes::default(),
            vec![0u8, 1, 2, 3],
        );
        // The face still takes its id — but the snapshot says "no metrics" rather
        // than handing back the idealized stand-in as if it were the font's own.
        assert_eq!(registry.face_metrics().len(), 1);
        assert!(registry.face_metrics().get(id).is_none());
    }

    #[test]
    fn is_object_safe() {
        let resolver: Box<dyn FontResolver> = Box::new(registry());
        let chain = alpha_beta_chain();
        let attributes = FontAttributes::default();
        assert!(
            resolver
                .resolve(&request(&chain, &attributes, "A"))
                .is_some()
        );
    }
}

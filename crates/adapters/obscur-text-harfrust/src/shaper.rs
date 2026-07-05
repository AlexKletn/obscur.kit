//! The [`HarfrustShaper`] adapter implementing [`TextShaper`] over HarfRust
//! (spec §5.10).

use std::sync::Arc;

use harfrust::{Direction, FontRef, Language, ShapeOptions, ShaperData, UnicodeBuffer};
use obscur_text::font::FontFace;
use obscur_text::shaping::{
    GlyphRun, LineMetrics, ShapedGlyph, ShapingRequest, TextDirection, TextShaper,
};
use obscur_text::span::TextRange;
use read_fonts::{FontRef as ReadFontRef, TableProvider};

/// A real [`TextShaper`] that shapes text with the HarfRust engine (spec §5.10).
///
/// The shaper holds the font **bytes** it shapes with. The primary path is
/// [`HarfrustShaper::from_bytes`]: the host wires one font and every request is
/// shaped against it (the request's own [`FontFace`] source is ignored, since it
/// may be a metadata-only placeholder). Alternatively [`without_font`] builds a
/// font-less shaper that falls back to per-request
/// [`FontSource::Memory`](obscur_text::font::FontSource::Memory) bytes carried on
/// the request's face.
///
/// Shaping is infallible: an empty run, absent font, or bytes HarfRust cannot
/// load all yield an empty [`GlyphRun`] rather than a panic, and identical
/// requests always produce identical output.
///
/// [`without_font`]: HarfrustShaper::without_font
#[derive(Debug, Clone, Default)]
pub struct HarfrustShaper {
    /// The wired host font bytes, or `None` for a font-less shaper that relies on
    /// per-request `Memory` bytes.
    font_bytes: Option<Arc<[u8]>>,
}

impl HarfrustShaper {
    /// Creates a shaper wired to a single host font from its raw `bytes`.
    ///
    /// Every [`shape`](TextShaper::shape) call uses this font, ignoring the
    /// request face's own source.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            font_bytes: Some(bytes.into()),
        }
    }

    /// Creates a font-less shaper that shapes only when a request's face carries
    /// [`FontSource::Memory`](obscur_text::font::FontSource::Memory) bytes.
    ///
    /// Without any font, [`shape`](TextShaper::shape) returns an empty run and
    /// [`can_shape`](TextShaper::can_shape) returns `false`.
    #[must_use]
    pub const fn without_font() -> Self {
        Self { font_bytes: None }
    }

    /// Returns the font bytes to shape `request` with: the wired host font if
    /// present, otherwise the request face's in-memory bytes.
    fn bytes_for<'a>(&'a self, request: &'a ShapingRequest) -> Option<&'a [u8]> {
        match self.font_bytes.as_deref() {
            Some(bytes) => Some(bytes),
            None => request.face().source().data(),
        }
    }
}

impl TextShaper for HarfrustShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        let text = request.text();
        let source = TextRange::new(0, text.len());
        let empty = || {
            GlyphRun::new(
                Vec::new(),
                request.direction(),
                request.font_size(),
                source,
            )
        };

        if text.is_empty() {
            return empty();
        }
        let Some(bytes) = self.bytes_for(request) else {
            return empty();
        };
        let Ok(font) = FontRef::new(bytes) else {
            return empty();
        };

        let shaper_data = ShaperData::new(&font);
        let shaper = shaper_data.shaper(&font).build();
        let units_per_em = shaper.units_per_em();
        if units_per_em <= 0 {
            return empty();
        }
        // Font units -> logical pixels. HarfRust returns positions in font units
        // (no scale set); scale by size/upem to honor the requested font size.
        let scale = request.font_size() / units_per_em as f32;

        // Seed the buffer one char at a time so each glyph's `cluster` is the
        // char's source **byte** offset, matching `ShapedGlyph::cluster`.
        let mut buffer = UnicodeBuffer::new();
        for (byte_index, ch) in text.char_indices() {
            // Cluster values are u32; saturate the (practically impossible)
            // >4 GiB run rather than wrap.
            buffer.add(ch, u32::try_from(byte_index).unwrap_or(u32::MAX));
        }
        buffer.set_direction(map_direction(request.direction()));
        if let Some(language) = request.language().and_then(Language::new) {
            buffer.set_language(language);
        }
        // Fill in any properties left unset (notably the script) by inspecting
        // the buffer contents; keeps the explicit direction/language.
        buffer.guess_segment_properties();

        let shaped = shaper.shape(buffer, ShapeOptions::new());
        let infos = shaped.glyph_infos();
        let positions = shaped.glyph_positions();
        // HarfRust returns glyphs already in pen (visual) order for the buffer's
        // direction, so this maps 1:1 without reordering.
        let glyphs = infos
            .iter()
            .zip(positions)
            .map(|(info, position)| {
                ShapedGlyph::new(
                    info.glyph_id,
                    info.cluster as usize,
                    position.x_advance as f32 * scale,
                )
                .with_offset(
                    position.x_offset as f32 * scale,
                    position.y_offset as f32 * scale,
                )
            })
            .collect();

        GlyphRun::new(glyphs, request.direction(), request.font_size(), source)
    }

    fn can_shape(&self, face: &FontFace) -> bool {
        self.font_bytes.is_some() || face.source().data().is_some()
    }

    fn line_metrics(&self, font_size: f32) -> LineMetrics {
        // Read the real vertical metrics from the wired host font; fall back to
        // the idealized default ratios when there is no font (or its tables are
        // unreadable), so a font-less shaper stays deterministic.
        self.font_bytes
            .as_deref()
            .and_then(|bytes| line_metrics_from_bytes(bytes, font_size))
            .unwrap_or_else(|| LineMetrics::idealized(font_size))
    }
}

/// Reads a font's real vertical [`LineMetrics`] from its `head` (units-per-em)
/// and `hhea` (ascender / descender / line-gap) tables, scaled to `font_size`.
///
/// Metrics are stored in font design units; scaling by `font_size / upem`
/// converts them to logical pixels. `hhea.descender` is negative below the
/// baseline, so its magnitude is taken to match [`LineMetrics`]'s positive
/// descent convention. Returns [`None`] when the bytes are not a loadable font,
/// the `head`/`hhea` tables are missing, or `units_per_em` is zero.
fn line_metrics_from_bytes(bytes: &[u8], font_size: f32) -> Option<LineMetrics> {
    let font = ReadFontRef::new(bytes).ok()?;
    let units_per_em = font.head().ok()?.units_per_em();
    if units_per_em == 0 {
        return None;
    }
    let hhea = font.hhea().ok()?;
    let scale = font_size / f32::from(units_per_em);
    let ascent = f32::from(hhea.ascender().to_i16()) * scale;
    let descent = f32::from(hhea.descender().to_i16()).abs() * scale;
    let line_gap = f32::from(hhea.line_gap().to_i16()) * scale;
    Some(LineMetrics::new(ascent, descent, line_gap))
}

/// Maps a domain [`TextDirection`] to the HarfRust [`Direction`] for horizontal
/// scripts.
const fn map_direction(direction: TextDirection) -> Direction {
    match direction {
        TextDirection::LeftToRight => Direction::LeftToRight,
        TextDirection::RightToLeft => Direction::RightToLeft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_text::font::{
        FontAttributes, FontFamily, FontMetrics, FontSource, GenericFamily,
    };

    /// Builds a tiny but valid TrueType font with three glyphs — `.notdef`,
    /// `'A'` (gid 1), and `'V'` (gid 2) — carrying a Unicode `cmap` (format 6)
    /// and real `hmtx` advances (500/600/550 units at 1000 upem). This is a
    /// genuine sfnt HarfRust can load and shape, not a magic-byte blob, so the
    /// test is fully hermetic (no system fonts).
    fn test_font() -> Vec<u8> {
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
            assert_eq!(t.len(), 54);
            t
        }

        fn hhea() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&800i16.to_be_bytes()); // ascender
            t.extend_from_slice(&(-200i16).to_be_bytes()); // descender
            t.extend_from_slice(&0i16.to_be_bytes()); // lineGap
            t.extend_from_slice(&[0u8; 24]); // advanceWidthMax .. metricDataFormat
            t.extend_from_slice(&3u16.to_be_bytes()); // numberOfHMetrics
            assert_eq!(t.len(), 36);
            t
        }

        fn maxp() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0000_5000u32.to_be_bytes()); // version 0.5
            t.extend_from_slice(&3u16.to_be_bytes()); // numGlyphs
            t
        }

        fn hmtx() -> Vec<u8> {
            let mut t = Vec::new();
            // longHorMetric { advanceWidth: u16, lsb: i16 } per glyph.
            for (advance, lsb) in [(500u16, 0i16), (600, 50), (550, 40)] {
                t.extend_from_slice(&advance.to_be_bytes());
                t.extend_from_slice(&lsb.to_be_bytes());
            }
            t
        }

        fn loca() -> Vec<u8> {
            // Short loca, numGlyphs + 1 = 4 entries, all zero: three empty glyphs.
            vec![0u8; 8]
        }

        fn cmap() -> Vec<u8> {
            // Format 6 (trimmed table) covering 0x41..=0x56 so 'A' -> 1, 'V' -> 2.
            const FIRST: u16 = 0x41; // 'A'
            const COUNT: u16 = 0x16; // 22 entries, last is 0x56 == 'V'
            let mut sub = Vec::new();
            sub.extend_from_slice(&6u16.to_be_bytes()); // format
            sub.extend_from_slice(&(10 + COUNT * 2).to_be_bytes()); // length
            sub.extend_from_slice(&0u16.to_be_bytes()); // language
            sub.extend_from_slice(&FIRST.to_be_bytes()); // firstCode
            sub.extend_from_slice(&COUNT.to_be_bytes()); // entryCount
            for code in FIRST..(FIRST + COUNT) {
                let gid = match code {
                    0x41 => 1u16, // 'A'
                    0x56 => 2u16, // 'V'
                    _ => 0u16,
                };
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
        }

        // Directory records must be tag-sorted.
        let tables: [(&[u8; 4], Vec<u8>); 7] = [
            (b"cmap", cmap()),
            (b"glyf", Vec::new()),
            (b"head", head()),
            (b"hhea", hhea()),
            (b"hmtx", hmtx()),
            (b"loca", loca()),
            (b"maxp", maxp()),
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
            // Pad each table to a 4-byte boundary; length records stay unpadded.
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

    fn face_with(source: FontSource) -> FontFace {
        FontFace::new(
            FontFamily::generic(GenericFamily::Serif),
            FontAttributes::default(),
            source,
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    fn system_face() -> FontFace {
        face_with(FontSource::system("placeholder"))
    }

    #[test]
    fn shapes_real_glyph_indices_and_scaled_advances() {
        let shaper = HarfrustShaper::from_bytes(test_font());
        let run = shaper.shape(&ShapingRequest::new("AV", system_face(), 32.0));

        assert_eq!(run.len(), 2);
        let a = run.glyph_at(0).unwrap();
        let v = run.glyph_at(1).unwrap();

        // Real, font-specific glyph indices from the cmap — NOT the code points.
        assert_eq!(a.glyph_id(), 1);
        assert_eq!(v.glyph_id(), 2);
        assert_ne!(a.glyph_id(), u32::from('A'));
        assert_ne!(v.glyph_id(), u32::from('V'));

        // Advances come from hmtx (600/550 units) scaled by 32/1000.
        assert!((a.x_advance() - 600.0 * 32.0 / 1000.0).abs() < 1e-4);
        assert!((v.x_advance() - 550.0 * 32.0 / 1000.0).abs() < 1e-4);
        assert!(a.x_advance() > 0.0 && v.x_advance() > 0.0);

        // Clusters are source byte offsets.
        assert_eq!(a.cluster(), 0);
        assert_eq!(v.cluster(), 1);

        assert_eq!(run.font_size(), 32.0);
        assert_eq!(run.source(), TextRange::new(0, 2));
    }

    #[test]
    fn clusters_track_multibyte_byte_offsets() {
        // "Aé": 'A' at byte 0, 'é' (two UTF-8 bytes) at byte 1. 'é' is not in the
        // cmap, so it maps to .notdef (glyph 0) — still a real glyph, not U+00E9.
        let shaper = HarfrustShaper::from_bytes(test_font());
        let run = shaper.shape(&ShapingRequest::new("Aé", system_face(), 16.0));
        assert_eq!(run.len(), 2);
        assert_eq!(run.glyph_at(0).unwrap().cluster(), 0);
        assert_eq!(run.glyph_at(1).unwrap().cluster(), 1);
        assert_eq!(run.glyph_at(1).unwrap().glyph_id(), 0); // .notdef / tofu
        assert_eq!(run.source(), TextRange::new(0, 3));
    }

    #[test]
    fn empty_text_yields_empty_run() {
        let shaper = HarfrustShaper::from_bytes(test_font());
        let run = shaper.shape(&ShapingRequest::new("", system_face(), 16.0));
        assert!(run.is_empty());
        assert_eq!(run.advance(), 0.0);
    }

    #[test]
    fn font_less_shaper_cannot_shape() {
        let shaper = HarfrustShaper::without_font();
        assert!(!shaper.can_shape(&system_face()));
        let run = shaper.shape(&ShapingRequest::new("AV", system_face(), 16.0));
        assert!(run.is_empty());
    }

    #[test]
    fn font_less_shaper_uses_request_memory_bytes() {
        let shaper = HarfrustShaper::without_font();
        let face = face_with(FontSource::memory(test_font()));
        assert!(shaper.can_shape(&face));
        let run = shaper.shape(&ShapingRequest::new("AV", face, 20.0));
        assert_eq!(run.len(), 2);
        assert_eq!(run.glyph_at(0).unwrap().glyph_id(), 1);
    }

    #[test]
    fn unloadable_bytes_yield_empty_run_without_panic() {
        let shaper = HarfrustShaper::from_bytes(vec![0u8, 1, 2, 3, 4, 5, 6, 7]);
        let run = shaper.shape(&ShapingRequest::new("AV", system_face(), 16.0));
        assert!(run.is_empty());
    }

    #[test]
    fn shaping_is_deterministic() {
        let shaper = HarfrustShaper::from_bytes(test_font());
        let request = ShapingRequest::new("AVA", system_face(), 18.0);
        assert_eq!(shaper.shape(&request), shaper.shape(&request));
    }

    #[test]
    fn can_shape_true_for_wired_font() {
        let shaper = HarfrustShaper::from_bytes(test_font());
        assert!(shaper.can_shape(&system_face()));
    }

    #[test]
    fn is_object_safe() {
        let shaper: Box<dyn TextShaper> = Box::new(HarfrustShaper::from_bytes(test_font()));
        assert!(shaper.can_shape(&system_face()));
    }

    #[test]
    fn line_metrics_read_the_loaded_fonts_scaled_vertical_metrics() {
        // The embedded font's hhea carries ascender 800, descender -200, lineGap
        // 0 at 1000 upem. At 32px the scale is 32/1000, so ascent = 25.6,
        // descent = 6.4 (magnitude), line_gap = 0 -> line_height = 32.
        let shaper = HarfrustShaper::from_bytes(test_font());
        let metrics = shaper.line_metrics(32.0);
        assert!((metrics.ascent() - 800.0 * 32.0 / 1000.0).abs() < 1e-4);
        assert!((metrics.descent() - 200.0 * 32.0 / 1000.0).abs() < 1e-4);
        assert_eq!(metrics.line_gap(), 0.0);
        assert!((metrics.line_height() - 32.0).abs() < 1e-4);
        // Baseline is the ascent, not the full em.
        assert!((metrics.baseline() - 25.6).abs() < 1e-4);
        assert_ne!(metrics.baseline(), 32.0);
    }

    #[test]
    fn font_less_line_metrics_fall_back_to_idealized_ratios() {
        // No wired font -> the idealized default (ascent 0.8·size, line_height
        // == size), preserving determinism.
        let shaper = HarfrustShaper::without_font();
        let metrics = shaper.line_metrics(20.0);
        assert_eq!(metrics.ascent(), 16.0); // 0.8 * 20
        assert_eq!(metrics.descent(), 4.0); // 0.2 * 20
        assert_eq!(metrics.line_height(), 20.0);
    }

    #[test]
    fn unloadable_bytes_line_metrics_fall_back_to_idealized() {
        let shaper = HarfrustShaper::from_bytes(vec![0u8, 1, 2, 3, 4, 5, 6, 7]);
        let metrics = shaper.line_metrics(16.0);
        assert_eq!(metrics.line_height(), 16.0);
        assert_eq!(metrics.baseline(), 12.8);
    }
}

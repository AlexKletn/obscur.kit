//! A font [`ResourceDecoder`] backed by `ttf-parser` (spec §5.18).
//!
//! [`FontDecoder`] validates encoded TTF/OTF bytes by parsing them into a
//! `ttf_parser::Face` — proving the sfnt tables (`head`, `hhea`, `maxp`, ...)
//! are well-formed and the metrics readable — and then keeps the original,
//! validated face bytes as the shared CPU representation. That is the natural
//! representation for a font: `ttf-parser` (and the text/shaping stack above it)
//! reads a `Face` zero-copy directly from these bytes, so re-wrapping them costs
//! nothing while the parse guarantees the downstream face will not fail (spec
//! §5.18 shaping/glyph caching).

use std::sync::Arc;

use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};

/// A [`ResourceDecoder`] that validates and admits font faces via `ttf-parser`
/// (spec §5.18).
///
/// This decoder supports only [`ResourceKind::Font`]. It parses the first face
/// (index 0) of the given bytes to confirm they form a usable face — malformed
/// bytes become [`ResourceError::DecodeFailed`] — then returns the validated
/// bytes unchanged as the [`DecodedResource`] CPU payload. Font collections are
/// admitted through their first face.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FontDecoder;

impl FontDecoder {
    /// Creates a font decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceDecoder for FontDecoder {
    fn supports(&self, kind: ResourceKind) -> bool {
        matches!(kind, ResourceKind::Font)
    }

    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        if kind != ResourceKind::Font {
            return Err(ResourceError::Unsupported(
                "font decoder handles only Font resources".into(),
            ));
        }

        // Parse to validate: proves head/hhea/maxp and the metrics are readable.
        let face = ttf_parser::Face::parse(bytes, 0)
            .map_err(|err| ResourceError::DecodeFailed(err.to_string().into()))?;

        // Touch a metric so the validated face is genuinely exercised, and reject
        // a degenerate face with no glyphs.
        let _units_per_em = face.units_per_em();
        if face.number_of_glyphs() == 0 {
            return Err(ResourceError::DecodeFailed("font has no glyphs".into()));
        }

        let metadata = ResourceMetadata::new()
            .with_byte_size(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
            .with_mime_type("font/sfnt");

        Ok(DecodedResource::new(Arc::from(bytes), metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal but valid sfnt TrueType font containing exactly the
    /// three tables `ttf-parser` requires to parse a face — `head`, `hhea`, and
    /// `maxp` — with real metrics (1000 units/em, ascender 800, one glyph). This
    /// is a genuine, spec-shaped fixture, not a magic-byte blob.
    fn minimal_font() -> Vec<u8> {
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
            t.extend_from_slice(&0u16.to_be_bytes()); // lowestRecPPEM
            t.extend_from_slice(&0i16.to_be_bytes()); // fontDirectionHint
            t.extend_from_slice(&0i16.to_be_bytes()); // indexToLocFormat
            t.extend_from_slice(&0i16.to_be_bytes()); // glyphDataFormat
            debug_assert_eq!(t.len(), 54);
            t
        }

        fn hhea() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // version
            t.extend_from_slice(&800i16.to_be_bytes()); // ascender
            t.extend_from_slice(&(-200i16).to_be_bytes()); // descender
            t.extend_from_slice(&0i16.to_be_bytes()); // lineGap
            t.extend_from_slice(&[0u8; 24]); // advanceWidthMax .. metricDataFormat
            t.extend_from_slice(&1u16.to_be_bytes()); // numberOfHMetrics
            debug_assert_eq!(t.len(), 36);
            t
        }

        fn maxp() -> Vec<u8> {
            let mut t = Vec::new();
            t.extend_from_slice(&0x0000_5000u32.to_be_bytes()); // version 0.5
            t.extend_from_slice(&1u16.to_be_bytes()); // numGlyphs
            debug_assert_eq!(t.len(), 6);
            t
        }

        // Table records must be present; keep them tag-sorted (head, hhea, maxp).
        let tables: [(&[u8; 4], Vec<u8>); 3] =
            [(b"head", head()), (b"hhea", hhea()), (b"maxp", maxp())];

        let num_tables = tables.len() as u16;
        let header_len = 12 + tables.len() * 16;
        let mut records = Vec::new();
        let mut body = Vec::new();
        let mut offset = header_len;
        for (tag, data) in &tables {
            records.extend_from_slice(*tag);
            records.extend_from_slice(&0u32.to_be_bytes()); // checksum (unchecked)
            records.extend_from_slice(&(offset as u32).to_be_bytes());
            records.extend_from_slice(&(data.len() as u32).to_be_bytes());
            offset += data.len();
            body.extend_from_slice(data);
        }

        let mut font = Vec::new();
        font.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // sfnt version (TrueType)
        font.extend_from_slice(&num_tables.to_be_bytes());
        font.extend_from_slice(&0u16.to_be_bytes()); // searchRange
        font.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
        font.extend_from_slice(&0u16.to_be_bytes()); // rangeShift
        font.extend_from_slice(&records);
        font.extend_from_slice(&body);
        font
    }

    #[test]
    fn supports_only_font() {
        let decoder = FontDecoder::new();
        assert!(decoder.supports(ResourceKind::Font));
        for kind in [
            ResourceKind::Image,
            ResourceKind::Svg,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!decoder.supports(kind));
        }
    }

    #[test]
    fn fixture_parses_with_ttf_parser() {
        // Prove the fixture is a real face and its metrics read back correctly.
        let font = minimal_font();
        let face = ttf_parser::Face::parse(&font, 0).expect("fixture is a valid face");
        assert_eq!(face.units_per_em(), 1000);
        assert_eq!(face.ascender(), 800);
        assert_eq!(face.descender(), -200);
        assert_eq!(face.number_of_glyphs(), 1);
    }

    #[test]
    fn decodes_valid_font_and_keeps_bytes() {
        let font = minimal_font();
        let decoded = FontDecoder::new()
            .decode(ResourceKind::Font, &font)
            .expect("font decodes");
        assert_eq!(&**decoded.data(), font.as_slice());
        assert_eq!(decoded.metadata().byte_size(), Some(font.len() as u64));
        assert_eq!(decoded.metadata().mime_type(), Some("font/sfnt"));
    }

    #[test]
    fn malformed_bytes_are_decode_failed() {
        let decoder = FontDecoder::new();
        let result = decoder.decode(ResourceKind::Font, &[0u8, 1, 2, 3, 4, 5, 6, 7]);
        assert!(matches!(result, Err(ResourceError::DecodeFailed(_))));
    }

    #[test]
    fn wrong_kind_is_unsupported() {
        let font = minimal_font();
        let result = FontDecoder::new().decode(ResourceKind::Image, &font);
        assert!(matches!(result, Err(ResourceError::Unsupported(_))));
    }
}

//! Read COLRv0 layers and resolve the default CPAL palette for color glyphs.

use obscur_scene::color::Color;
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId};

pub(super) fn layers(
    face: &FontRef<'_>,
    glyph: GlyphId,
) -> Result<impl Iterator<Item = Result<(GlyphId, Option<Color>), &'static str>>, &'static str> {
    let colr = face.colr().map_err(|_| "invalid COLR table")?;
    let range = colr
        .v0_base_glyph(glyph)
        .map_err(|_| "invalid COLRv0 base glyph")?
        .ok_or("missing COLRv0 base glyph")?;
    let palette = Palette::new(face)?;
    Ok(range.map(move |index| {
        let (glyph, palette_index) = colr.v0_layer(index).map_err(|_| "invalid COLRv0 layer")?;
        Ok((GlyphId::from(glyph), palette.color(palette_index)?))
    }))
}

pub(super) struct Palette<'a> {
    colors: &'a [skrifa::raw::tables::cpal::ColorRecord],
    first: usize,
    count: u16,
}

impl<'a> Palette<'a> {
    pub(super) fn new(face: &FontRef<'a>) -> Result<Self, &'static str> {
        let cpal = face.cpal().map_err(|_| "invalid CPAL table")?;
        let first = cpal
            .color_record_indices()
            .first()
            .ok_or("missing default CPAL palette")?
            .get();
        let colors = cpal
            .color_records_array()
            .transpose()
            .map_err(|_| "invalid CPAL colors")?
            .ok_or("missing CPAL colors")?;
        Ok(Self {
            colors,
            first: usize::from(first),
            count: cpal.num_palette_entries(),
        })
    }

    pub(super) fn color(&self, index: u16) -> Result<Option<Color>, &'static str> {
        if index == u16::MAX {
            return Ok(None);
        }
        if index >= self.count {
            return Err("invalid COLR palette index");
        }
        let entry = self
            .colors
            .get(self.first + usize::from(index))
            .ok_or("missing COLR palette entry")?;
        let channel = |value| f32::from(value) / 255.0;
        Ok(Some(Color::rgba(
            channel(entry.red),
            channel(entry.green),
            channel(entry.blue),
            channel(entry.alpha),
        )))
    }
}

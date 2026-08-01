#![doc = include_str!("line_truncation.doc/_module.md")]

use crate::shaping::ShapedGlyph;

use super::TextOverflow;
use super::packed_line::PackedLine;
use super::tagged_glyph::TaggedGlyph;

#[doc = include_str!("line_truncation.doc/truncate_lines.md")]
pub(super) fn truncate_lines(
    lines: &mut [PackedLine],
    max_width: f32,
    overflow: TextOverflow,
    ellipsis_glyph: Option<ShapedGlyph>,
) {
    for packed in lines {
        if packed.width() > max_width {
            truncate_line(&mut packed.glyphs, max_width, overflow, ellipsis_glyph);
        }
    }
}

#[doc = include_str!("line_truncation.doc/truncate_line.md")]
fn truncate_line(
    glyphs: &mut Vec<TaggedGlyph>,
    max_width: f32,
    overflow: TextOverflow,
    ellipsis_glyph: Option<ShapedGlyph>,
) {
    // Only `Ellipsis` reserves and later appends a glyph; `Clip` (and
    // `Ellipsis` without a supplied glyph) truncates without adding anything.
    let ellipsis = match overflow {
        TextOverflow::Ellipsis => ellipsis_glyph,
        _ => None,
    };
    let limit = max_width - ellipsis.map_or(0.0, |g| g.x_advance());

    // Keep the largest prefix whose cumulative advance fits `limit`.
    let mut kept = 0_usize;
    let mut width = 0.0_f32;
    for tagged in glyphs.iter() {
        let next = width + tagged.glyph.x_advance();
        if next > limit {
            break;
        }
        width = next;
        kept += 1;
    }

    // The carrier lends the ellipsis its font/direction/level and cluster: the
    // last kept glyph, or the first glyph when nothing fit.
    let carrier = glyphs
        .get(kept.wrapping_sub(1))
        .or_else(|| glyphs.first())
        .copied();
    glyphs.truncate(kept);
    if let (Some(glyph), Some(carrier)) = (ellipsis, carrier) {
        let ellipsis =
            ShapedGlyph::new(glyph.glyph_id(), carrier.glyph.cluster(), glyph.x_advance())
                .with_offset(glyph.x_offset(), glyph.y_offset());
        glyphs.push(TaggedGlyph {
            glyph: ellipsis,
            ..carrier
        });
    }
}

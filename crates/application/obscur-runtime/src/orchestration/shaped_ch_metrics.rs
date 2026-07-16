//! Supplies the computed-value step with a real `0` advance:
//! [`ShapedChMetrics`].
//!
//! This is the runtime's implementation of `obscur-style`'s [`ChMetrics`] port —
//! the adapter that lets the resolver *ask* for the one number it cannot compute.
//! It shapes the `0` glyph through the same [`ParagraphShaper`] the measure and
//! compose passes use, so a `20ch` box, the twenty characters placed in it, and
//! the pixels painted for both are all measured against one font.
//!
//! # Why this exists at all
//!
//! `ch` was the last relative unit to die at the layout↔paint seam. Every other
//! unit folds to `px` at computed-value time, so paint reads pixels; `ch` stayed
//! symbolic on the `ComputedStyle` because the resolver had no shaper. Layout
//! could measure it and did, paint could not and read `0` — so
//! `border-width: 1ch` reserved a 24px ring and painted nothing. Handing the
//! resolver this port moves the measurement to the same place every other unit is
//! resolved, which is what makes the two phases agree.

use obscur_style::computed::{ChMetrics, ComputedStyle};
use obscur_text::shaping::ParagraphShaper;

use super::font_families::families_for;
use super::layout_coordinator::{font_size_px, measure_ch_width};
use super::text_attributes::attributes_for;

/// A [`ChMetrics`] port backed by a live [`ParagraphShaper`].
///
/// Borrows the shaper rather than owning one: the runtime already owns exactly
/// one (the scene coordinator's), and `ch` must be measured with *that* shaper —
/// a second one could disagree, which is the class of bug this port closes.
///
/// Not [`Debug`]: [`ParagraphShaper`] is not, which is precisely why `ChMetrics`
/// does not require it of an implementor.
#[derive(Clone, Copy)]
pub struct ShapedChMetrics<'a> {
    /// The shaper every other text measurement in the frame goes through.
    paragraph: &'a ParagraphShaper,
}

impl<'a> ShapedChMetrics<'a> {
    /// Wraps `paragraph` as the resolver's `ch` measurement port.
    #[must_use]
    pub const fn new(paragraph: &'a ParagraphShaper) -> Self {
        Self { paragraph }
    }
}

impl ChMetrics for ShapedChMetrics<'_> {
    /// Shapes `0` in the face `font` selects, at the size `font` resolved.
    ///
    /// Reads exactly the four properties the port's contract promises are present
    /// — `font-family` (via [`families_for`]), `font-weight` / `font-style` (via
    /// [`attributes_for`]), and `font-size` (via [`font_size_px`]) — and nothing
    /// else, because nothing else is resolved yet when the resolver calls this.
    ///
    /// These are the same three helpers the layout measure pass and the scene
    /// compose pass use to build *their* shaping requests, which is what makes the
    /// `ch` answer consistent with the text: they are not re-derived here.
    ///
    /// # Standing limitation
    ///
    /// `ch` **does** follow `font-family`: the node's families are passed below and
    /// the resolver's fallback chain picks the face, so a monospace node measures
    /// its own `0` (Noto Sans Mono's 600/1000 vs Noto Sans's 572/1000 — measurably
    /// different at every size). Only the `FontFace` *carrier* is ignored.
    ///
    /// What is still open is narrower and worth stating exactly, because it is the
    /// seam this port exists to close: a family that resolves to **no registered
    /// face** — `font-family: Helvetica` with neither Helvetica nor a fallback
    /// registered — measures nothing. `zero_advance` answers `None`, layout then
    /// takes `ch_base`'s 0.5em substitute, and paint leaves `Ch` symbolic and reads
    /// 0. Layout and paint disagree again, exactly as they did for `em` before 4-A.
    /// This coincides with a general fallback gap (such a node renders no glyphs at
    /// all), so `ch` inherits it rather than causing it — but the seam is live for
    /// an unresolvable family even though every production driver supplies metrics.
    fn zero_advance(&self, font: &ComputedStyle) -> Option<f32> {
        measure_ch_width(
            self.paragraph,
            &families_for(Some(font)),
            &attributes_for(Some(font)),
            font_size_px(font),
        )
    }
}

//! `GridTrack`: a style-local CSS-grid track sizing function.
//!
//! [`GridTrack`] is the `obscur-style` model of a single `grid-template-columns`
//! / `grid-template-rows` track (spec §4.1). It is deliberately **style-local**:
//! `obscur-style` is a pure domain crate that does not depend on `obscur-layout`,
//! so authored templates are modelled here in style vocabulary and the runtime
//! converts each [`GridTrack`] to the layout crate's own `TrackSize` when it
//! bridges a computed style into a layout input.
//!
//! Unlike the layout crate's already-pixel-resolved track, a [`GridTrack::Fixed`]
//! keeps its full [`Length`] (so `px`, `%`, `em`, … survive to the compute step
//! and resolve against a `LengthContext` in the runtime); the `fr` fraction,
//! `auto`, and `minmax(min, max)` mirror the layout track model exactly.

use crate::property::Length;

/// The sizing function for one grid track (a column or a row), in style terms.
///
/// This is the style-side twin of `obscur_layout`'s `TrackSize`; the runtime maps
/// one to the other. The four variants are the track sizing functions this stack
/// supports:
///
/// * [`Fixed`](GridTrack::Fixed) — a fixed [`Length`] (`100px`, `50%`, `2em`, …).
///   The length is resolved to pixels at the compute step, not here.
/// * [`Fr`](GridTrack::Fr) — a flexible fraction (`fr`): shares leftover space.
/// * [`Auto`](GridTrack::Auto) — content-sized.
/// * [`MinMax`](GridTrack::MinMax) — a `minmax(min, max)` clamp; each bound is
///   itself a [`GridTrack`].
/// * [`AutoRepeat`](GridTrack::AutoRepeat) — a `repeat(auto-fill | auto-fit,
///   <track-list>)` auto-repetition. Its repetition count depends on the grid
///   container's content extent, which is unknown at CSS-lowering time, so the
///   pattern is carried symbolically here (in author order among the other
///   tracks) and expanded by the layout algorithm.
///
/// Because [`Fixed`](GridTrack::Fixed) carries an `f32`-backed [`Length`] and
/// [`Fr`](GridTrack::Fr) an `f32`, this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    /// A fixed track length (`100px`, `50%`, `2em`, …), resolved to pixels later.
    Fixed(Length),
    /// A flexible fraction (`fr`): shares leftover space proportionally.
    Fr(f32),
    /// Content-sized: as large as the biggest item placed in the track.
    Auto,
    /// `minmax(min, max)`: sized by `max`, floored at `min`. Each bound is itself
    /// a [`GridTrack`].
    MinMax(Box<GridTrack>, Box<GridTrack>),
    /// A `repeat(auto-fill | auto-fit, <track-list>)` auto-repetition: `fill` is
    /// `true` for `auto-fill` (empty tracks kept), `false` for `auto-fit` (empty
    /// tracks collapse). `tracks` is one copy of the repeated pattern. The
    /// repetition count is resolved against the container size inside layout.
    AutoRepeat {
        /// `true` for `auto-fill`, `false` for `auto-fit`.
        fill: bool,
        /// One copy of the repeated track pattern.
        tracks: Box<[GridTrack]>,
    },
}

impl GridTrack {
    /// Builds a [`MinMax`](GridTrack::MinMax) track from a `min` and `max` bound,
    /// boxing each so callers need not.
    #[must_use]
    pub fn minmax(min: GridTrack, max: GridTrack) -> Self {
        GridTrack::MinMax(Box::new(min), Box::new(max))
    }
}

#[cfg(test)]
mod tests {
    use super::GridTrack;
    use crate::property::Length;

    #[test]
    fn minmax_boxes_bounds() {
        let t = GridTrack::minmax(GridTrack::Fixed(Length::px(100.0)), GridTrack::Fr(1.0));
        assert_eq!(
            t,
            GridTrack::MinMax(
                Box::new(GridTrack::Fixed(Length::px(100.0))),
                Box::new(GridTrack::Fr(1.0))
            )
        );
    }

    #[test]
    fn variants_are_equatable_and_clonable() {
        assert_eq!(GridTrack::Auto.clone(), GridTrack::Auto);
        assert_ne!(GridTrack::Fr(1.0), GridTrack::Fr(2.0));
        assert_ne!(GridTrack::Fixed(Length::px(10.0)), GridTrack::Auto);
    }
}

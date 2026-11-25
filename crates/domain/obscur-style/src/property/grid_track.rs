#![doc = include_str!("../../docs/api/property/grid_track.doc/_module.md")]

use crate::property::{Length, LengthExpr};

#[doc = include_str!("../../docs/api/property/grid_track.doc/GridTrack.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    /// A fixed track length (`100px`, `50%`, `2em`, …), resolved to pixels later.
    Fixed(Length),
    /// A flexible fraction (`fr`): shares leftover space proportionally.
    Fr(f32),
    /// Content-sized: as large as the biggest item placed in the track.
    Auto,
    /// `min-content`: the largest of the items' min-content contributions — as
    /// narrow as the track can be without its items overflowing (CSS Grid §7.2.3).
    MinContent,
    /// `max-content`: the largest of the items' max-content contributions — as
    /// wide as the items want given unbounded space (CSS Grid §7.2.3).
    MaxContent,
    /// An intrinsic grid track with a length or percentage growth cap.
    FitContent(Length),
    /// An intrinsic grid track capped by a CSS math expression.
    FitContentMath(LengthExpr),
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
        /// Names on each boundary of the repeated pattern, when authored.
        line_names: Box<[Box<[Box<str>]>]>,
    },
}

impl GridTrack {
    /// Builds a [`MinMax`](GridTrack::MinMax) track from a `min` and `max` bound,
    /// boxing each so callers need not.
    #[must_use]
    pub fn minmax(min: GridTrack, max: GridTrack) -> Self {
        GridTrack::MinMax(Box::new(min), Box::new(max))
    }

    #[doc = include_str!("../../docs/api/property/grid_track.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Fixed(length) | Self::FitContent(length) => length.is_ch(),
            Self::FitContentMath(limit) => limit.uses_ch(),
            Self::MinMax(min, max) => min.uses_ch() || max.uses_ch(),
            Self::AutoRepeat { tracks, .. } => tracks.iter().any(Self::uses_ch),
            Self::Fr(_) | Self::Auto | Self::MinContent | Self::MaxContent => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GridTrack;
    use crate::property::Length;

    #[test]
    fn uses_ch_recurses_through_minmax_and_auto_repeat() {
        assert!(GridTrack::Fixed(Length::Ch(20.0)).uses_ch());
        assert!(
            GridTrack::minmax(GridTrack::Fixed(Length::Ch(20.0)), GridTrack::Fr(1.0)).uses_ch()
        );
        assert!(GridTrack::minmax(GridTrack::Auto, GridTrack::Fixed(Length::Ch(4.0))).uses_ch());
        // `repeat(auto-fill, 20ch)`: the pattern is where the `ch` hides.
        assert!(
            GridTrack::AutoRepeat {
                line_names: Box::from([]),
                fill: true,
                tracks: vec![GridTrack::Fixed(Length::Ch(20.0))].into_boxed_slice(),
            }
            .uses_ch()
        );
    }

    #[test]
    fn uses_ch_is_false_for_tracks_without_one() {
        assert!(!GridTrack::Auto.uses_ch());
        assert!(!GridTrack::Fr(1.0).uses_ch());
        assert!(!GridTrack::Fixed(Length::px(100.0)).uses_ch());
        assert!(
            !GridTrack::Fixed(Length::Em(2.0)).uses_ch(),
            "`em` is not `ch`"
        );
        assert!(!GridTrack::minmax(GridTrack::Auto, GridTrack::Fr(1.0)).uses_ch());
        assert!(
            !GridTrack::AutoRepeat {
                line_names: Box::from([]),
                fill: false,
                tracks: vec![GridTrack::Fixed(Length::px(80.0))].into_boxed_slice(),
            }
            .uses_ch()
        );
    }

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

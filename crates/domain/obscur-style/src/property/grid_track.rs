#![doc = include_str!("grid_track.doc/_module.md")]

use crate::property::Length;

#[doc = include_str!("grid_track.doc/GridTrack.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    #[doc = include_str!("grid_track.doc/Fixed.md")]
    Fixed(Length),
    #[doc = include_str!("grid_track.doc/Fr.md")]
    Fr(f32),
    #[doc = include_str!("grid_track.doc/Auto.md")]
    Auto,
    #[doc = include_str!("grid_track.doc/MinContent.md")]
    MinContent,
    #[doc = include_str!("grid_track.doc/MaxContent.md")]
    MaxContent,
    #[doc = include_str!("grid_track.doc/MinMax.md")]
    MinMax(Box<GridTrack>, Box<GridTrack>),
    #[doc = include_str!("grid_track.doc/AutoRepeat.md")]
    AutoRepeat {
        #[doc = include_str!("grid_track.doc/fill.md")]
        fill: bool,
        #[doc = include_str!("grid_track.doc/tracks.md")]
        tracks: Box<[GridTrack]>,
    },
}

impl GridTrack {
    #[doc = include_str!("grid_track.doc/minmax.md")]
    #[must_use]
    pub fn minmax(min: GridTrack, max: GridTrack) -> Self {
        GridTrack::MinMax(Box::new(min), Box::new(max))
    }

    #[doc = include_str!("grid_track.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        match self {
            Self::Fixed(length) => length.is_ch(),
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

//! `TrackSize`: the size of a single CSS-grid row or column track.

use crate::model::Dimension;

use super::TrackLimit;

#[doc = include_str!("../../../docs/api/algorithm/grid/track_size.doc/TrackSize.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSize {
    /// A resolved pixel length (already unit-resolved by the CSS layer).
    Fixed(f32),
    /// A fraction of the container's content extent along this axis, stored as a
    /// `0..=1` fraction (`30%` → `0.30`). Resolved to `fraction * axis_size`
    /// inside the algorithm — where the axis extent is known — and, like
    /// [`Fixed`](TrackSize::Fixed), consumes its space before `fr` distribution.
    Percent(f32),
    /// A flexible fraction (`fr`); shares leftover space proportionally.
    Fr(f32),
    /// Content-sized: as large as the biggest item placed in the track.
    Auto,
    /// `min-content`: the largest **min-content** contribution among the items
    /// placed in the track — each measured under an offer of no space.
    MinContent,
    /// `max-content`: the largest **max-content** contribution among the items
    /// placed in the track — each measured under an unbounded offer.
    MaxContent,
    /// A max-content growth limit capped by a dimension, with an intrinsic minimum.
    FitContent(Dimension),
    /// An intrinsic track whose growth cap is evaluated against the grid axis.
    FitContentMath(Box<TrackLimit>),
    #[doc = include_str!("../../../docs/api/algorithm/grid/track_size.doc/MinMax.md")]
    MinMax(Box<TrackSize>, Box<TrackSize>),
}

impl TrackSize {
    #[doc = include_str!("../../../docs/api/algorithm/grid/track_size.doc/minmax.md")]
    #[must_use]
    pub fn minmax(min: TrackSize, max: TrackSize) -> Self {
        TrackSize::MinMax(Box::new(min), Box::new(max))
    }
}

#[cfg(test)]
mod tests {
    use super::TrackSize;

    #[test]
    fn minmax_boxes_bounds() {
        let t = TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0));
        assert_eq!(
            t,
            TrackSize::MinMax(
                Box::new(TrackSize::Fixed(100.0)),
                Box::new(TrackSize::Fr(1.0))
            )
        );
    }

    #[test]
    fn variants_are_equatable_and_clonable() {
        let a = TrackSize::Auto;
        assert_eq!(a.clone(), TrackSize::Auto);
        assert_ne!(TrackSize::Fr(1.0), TrackSize::Fr(2.0));
        assert_ne!(TrackSize::Fixed(10.0), TrackSize::Auto);
        assert_eq!(TrackSize::Percent(0.30).clone(), TrackSize::Percent(0.30));
        assert_ne!(TrackSize::Percent(0.30), TrackSize::Percent(0.50));
        assert_ne!(TrackSize::Percent(0.50), TrackSize::Fixed(0.50));
    }
}

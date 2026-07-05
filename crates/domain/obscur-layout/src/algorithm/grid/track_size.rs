//! `TrackSize`: the size of a single CSS-grid row or column track.

/// The sizing function for one grid track (a column or a row).
///
/// A track's size is resolved by the grid algorithm from the container's
/// available space, the other tracks, and — for content-sized tracks — the items
/// placed in it (spec §4.1 grid track sizing). This enum models the four track
/// sizing functions this crate supports:
///
/// * [`Fixed`](TrackSize::Fixed) — a resolved pixel length. The CSS side resolves
///   any unit (`px`, `em`, `%`, …) to a plain `f32` *before* constructing this
///   value; the layout algorithm treats it as an absolute logical-pixel length.
/// * [`Fr`](TrackSize::Fr) — a flexible fraction (`fr` unit). Flexible tracks
///   share the space left after fixed and content-sized tracks, in proportion to
///   their fraction (analogous to `flex-grow`).
/// * [`Auto`](TrackSize::Auto) — content-sized. The track is sized to the largest
///   content size among the items placed in it.
/// * [`MinMax`](TrackSize::MinMax) — a `minmax(min, max)` clamp. The track is
///   sized by its `max` function but is never smaller than its `min` function;
///   `minmax(100px, 1fr)` grows flexibly from a `100px` floor.
///
/// # Determinism
///
/// `TrackSize` carries no interior mutability and resolves purely from its own
/// value plus the container/content sizes, so track sizing is deterministic.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSize {
    /// A resolved pixel length (already unit-resolved by the CSS layer).
    Fixed(f32),
    /// A flexible fraction (`fr`); shares leftover space proportionally.
    Fr(f32),
    /// Content-sized: as large as the biggest item placed in the track.
    Auto,
    /// `minmax(min, max)`: sized by `max`, floored at `min`.
    ///
    /// Each bound is itself a [`TrackSize`] (conventionally one of
    /// [`Fixed`](TrackSize::Fixed), [`Fr`](TrackSize::Fr), or
    /// [`Auto`](TrackSize::Auto)); an `fr` bound is only meaningful as the `max`.
    MinMax(Box<TrackSize>, Box<TrackSize>),
}

impl TrackSize {
    /// Builds a [`MinMax`](TrackSize::MinMax) track from a `min` and `max` bound.
    ///
    /// Convenience so callers need not box the bounds by hand:
    /// `TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0))`.
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
    }
}

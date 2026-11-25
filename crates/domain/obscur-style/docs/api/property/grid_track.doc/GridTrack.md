The sizing function for one grid track (a column or a row), in style terms.

This is the style-side twin of `obscur_layout`'s `TrackSize`; the runtime maps
one to the other. The variants are the track sizing functions this stack
supports:

* [`Fixed`](GridTrack::Fixed) — a fixed [`Length`] (`100px`, `50%`, `2em`, …).
  The length is resolved to pixels at the compute step, not here.
* [`Fr`](GridTrack::Fr) — a flexible fraction (`fr`): shares leftover space.
* [`Auto`](GridTrack::Auto) — content-sized.
* [`MinContent`](GridTrack::MinContent) / [`MaxContent`](GridTrack::MaxContent)
  — the intrinsic track sizing keywords.
* [`MinMax`](GridTrack::MinMax) — a `minmax(min, max)` clamp; each bound is
  itself a [`GridTrack`].
* [`AutoRepeat`](GridTrack::AutoRepeat) — a `repeat(auto-fill | auto-fit,
  <track-list>)` auto-repetition. Its repetition count depends on the grid
  container's content extent, which is unknown at CSS-lowering time, so the
  pattern is carried symbolically here (in author order among the other
  tracks) and expanded by the layout algorithm.

Because [`Fixed`](GridTrack::Fixed) carries an `f32`-backed [`Length`] and
[`Fr`](GridTrack::Fr) an `f32`, this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].

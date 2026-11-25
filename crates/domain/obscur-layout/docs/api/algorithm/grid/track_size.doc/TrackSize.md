The sizing function for one grid track (a column or a row).

A track's size is resolved by the grid algorithm from the container's
available space, the other tracks, and — for content-sized tracks — the items
placed in it (spec §4.1 grid track sizing). This enum models the track sizing
functions this crate supports:

* [`Fixed`](TrackSize::Fixed) — a resolved pixel length. The CSS side resolves
  any absolute or relative unit (`px`, `em`, `vw`, …) to a plain `f32` *before*
  constructing this value; the layout algorithm treats it as an absolute
  logical-pixel length.
* [`Percent`](TrackSize::Percent) — a percentage of the container's content
  extent along this axis (`30%` → `0.30`). Unlike a fixed unit, a percentage's
  basis (the grid container's content width for columns / content height for
  rows) is only known *inside* the algorithm, so it is carried symbolically to
  here and resolved against that extent at track-sizing time. Like a fixed
  track it is inflexible: it consumes its space before `fr` distribution.
* [`Fr`](TrackSize::Fr) — a flexible fraction (`fr` unit). Flexible tracks
  share the space left after fixed, percentage, and content-sized tracks, in
  proportion to their fraction (analogous to `flex-grow`).
* [`Auto`](TrackSize::Auto) — content-sized. The track is sized to the largest
  content size among the items placed in it.
* [`MinContent`](TrackSize::MinContent) / [`MaxContent`](TrackSize::MaxContent)
  — the intrinsic track sizing keywords. Like [`Auto`](TrackSize::Auto) they
  are content-sized, but they name *which* content size: the smallest the items
  can be without overflowing, or the size they want given unbounded space (CSS
  Sizing 3 §4.1). Unlike `auto` they do not stretch to absorb leftover space
  (CSS Grid §11.8).
* [`MinMax`](TrackSize::MinMax) — a `minmax(min, max)` clamp. The track is
  sized by its `max` function but is never smaller than its `min` function;
  `minmax(100px, 1fr)` grows flexibly from a `100px` floor. A [`Percent`]
  bound resolves against the axis extent just like a standalone percentage.

# Determinism

`TrackSize` carries no interior mutability and resolves purely from its own
value plus the container/content sizes, so track sizing is deterministic.

[`Percent`]: TrackSize::Percent

Expands a `repeat(…)` component into [`GridTrack`]s.

An integer `repeat(N, <tracks>)` is expanded in place into `N` flat copies of
its inner track list (`N` a positive integer clamped to [`MAX_REPEAT`]). A
`repeat(auto-fill | auto-fit, <tracks>)` cannot be resolved without the
container size, so it is carried symbolically as a single
[`GridTrack::AutoRepeat`] (the layout algorithm computes the repetition count
from the container's content extent); the `auto-fill` / `auto-fit` distinction
is preserved in its `fill` flag. Returns [`None`] on a missing count, an empty
inner list, or an unparseable inner track.

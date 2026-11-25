A `repeat(auto-fill | auto-fit, <track-list>)` auto-repetition (CSS §4.1).

Unlike an integer `repeat(N, …)` — which the CSS layer expands to `N` flat
copies up front — an auto-repeat's repetition count depends on the grid
container's content extent along the axis, which is only known inside the
layout algorithm. So the pattern is carried symbolically here and expanded by
[`TrackList::expand`](super::TrackList::expand) at track-sizing time: as many
whole copies of `tracks` as fit the content extent.

The pattern `tracks` are expected to be fixed- or percentage-sized (an `fr` or
`auto` track has no definite size to fit repetitions against); a non-definite
pattern track contributes `0` to the fitted extent (a documented fallback) so
the count stays finite and deterministic.

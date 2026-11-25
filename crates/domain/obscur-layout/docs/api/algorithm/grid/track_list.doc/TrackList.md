One axis of a grid template: leading explicit tracks, an optional auto-repeat,
and trailing explicit tracks (CSS §4.1 `grid-template-columns` / `-rows`).

A plain track list (no `repeat(auto-fill|auto-fit, …)`) is modelled with all
tracks in [`leading`](Self::leading) and no [`auto_repeat`](Self::auto_repeat)
— [`TrackList::from_tracks`] builds exactly that, so a template without an
auto-repeat expands **byte-identically** to its explicit track vector.

When an [`AutoRepeat`] is present the list reads
`leading ++ (N × pattern) ++ trailing`, where the repetition count `N` is
computed from the container's content extent along the axis at track-sizing
time by [`expand`](Self::expand). Only one auto-repeat is representable per
axis (as in CSS).

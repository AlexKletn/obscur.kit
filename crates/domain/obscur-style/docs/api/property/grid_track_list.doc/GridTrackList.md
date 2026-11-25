A grid container's `grid-template-columns` / `grid-template-rows` value: the
flat, author-order list of [`GridTrack`] sizing functions together with the
`[name]` line names that were interleaved with them (spec §4.1, §4.13).

A track list names lines as well as sizing tracks: `[sidebar-start] 200px
[sidebar-end main-start] 1fr [main-end]` sizes two tracks (`200px`, `1fr`) and
names the three grid lines around them. The sizes live in
[`tracks`](GridTrackList::tracks) and the names in
[`line_names`](GridTrackList::line_names), one bucket per grid line — so
`line_names.len() == tracks.len() + 1` whenever any name was authored (a track
list with no `[name]` group carries an **empty** `line_names`, keeping the
no-names path allocation-free and byte-identical to the pre-named-lines value).

A line can carry several names (`[a b]`), so each bucket is itself a list. A
child's `grid-column: <name>` is resolved against this list in the runtime, the
same way `grid-area: <name>` is resolved against the container's
`grid-template-areas`.

# Determinism

`GridTrackList` owns its data with no interior mutability, so a container's
template is a deterministic value with a stable [`PartialEq`]. Because
[`GridTrack`] carries `f32` data this derives [`PartialEq`] but not [`Eq`].

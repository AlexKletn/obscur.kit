Converts a single node's computed grid template into an `obscur_layout`
[`GridTemplate`], or [`None`] when the node authored neither track list.

The column and row track lists are read from [`PropertyId::GridTemplateColumns`]
/ [`PropertyId::GridTemplateRows`] (each a [`PropertyValue::GridTracks`]) and
each [`GridTrack`] is mapped to a [`TrackSize`] via [`track_size_of`]. An empty
`rows` list yields an all-implicit-row (auto) grid, the common columns-only
template.

Maps one style [`GridTrack`] into an `obscur_layout` [`TrackSize`].

A [`GridTrack::Fixed`] length is resolved to logical pixels against `ctx`; a
bare percentage — its basis is the not-yet-known container content size —
becomes a [`TrackSize::Percent`] fraction the algorithm resolves later, and
`auto` (no numeric value) degrades to [`TrackSize::Auto`]. `fr`, `auto`,
`min-content` / `max-content`, and `minmax` map structurally, recursing on the
`minmax` bounds.

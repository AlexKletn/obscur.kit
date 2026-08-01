`GridTrack`: a style-local CSS-grid track sizing function.

[`GridTrack`] is the `obscur-style` model of a single `grid-template-columns`
/ `grid-template-rows` track (spec §4.1). It is deliberately **style-local**:
`obscur-style` is a pure domain crate that does not depend on `obscur-layout`,
so authored templates are modelled here in style vocabulary and the runtime
converts each [`GridTrack`] to the layout crate's own `TrackSize` when it
bridges a computed style into a layout input.

Unlike the layout crate's already-pixel-resolved track, a [`GridTrack::Fixed`]
keeps its full [`Length`] (so `px`, `%`, `em`, … survive to the compute step
and resolve against a `LengthContext` in the runtime); the `fr` fraction,
`auto`, the content keywords, and `minmax(min, max)` mirror the layout track
model exactly.

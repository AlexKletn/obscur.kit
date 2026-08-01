Lowers a `grid-auto-columns` / `grid-auto-rows` value into a
[`PropertyValue::GridTracks`] track list — the implicit-track sizing functions.

The track grammar is identical to `grid-template-*` (a `<length>`, an `<fr>`,
`auto`, or `minmax(min, max)`, with an integer `repeat(N, …)` expanded in
place), and it *reuses* [`tracks_from_components`]. The one difference: an
`auto-fill` / `auto-fit` `repeat(…)` is **invalid** for auto-tracks (there is
no container basis for a repetition count here), so a lowered
[`GridTrack::AutoRepeat`] degrades the declaration to an unsupported-feature
diagnostic (returns [`None`]) rather than being sized against a meaningless
repeat. Returns [`None`] on an empty list, exactly like `grid-template-*`.

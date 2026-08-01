Parses a `grid-template-columns` / `-rows` value into its flat [`GridTrack`]
list **and** the `[name]` names on each grid line (spec §4.1, §4.13).

This walks the same track grammar as [`tracks_from_components`], but also
threads a per-line name accumulator: a [`ParsedValue::LineNames`] group names
the line at the current boundary (before the next track), so
`[a] 100px [b] 1fr [c]` yields tracks `[100px, 1fr]` and line names
`[[a], [b], [c]]` — one bucket per grid line (`tracks.len() + 1`). Names
inside a `repeat(…)` are not tracked (its tracks expand, its names are
dropped) — a documented limitation. Returns [`None`] on any unparseable track.

A grid track list, in author order: the value of `grid-template-columns`
or `grid-template-rows`. The [`GridTrackList`] carries the [`GridTrack`]
sizing functions (`100px`, `1fr`, `auto`, `minmax(…)`; `repeat(…)` already
expanded to a flat list) plus the `[name]` names on each grid line. The
runtime converts each [`GridTrack`] to `obscur_layout`'s `TrackSize`, and
resolves a child's named-line placement against the line names, when it
bridges the computed style into a grid layout input (spec §4.1, §4.13).

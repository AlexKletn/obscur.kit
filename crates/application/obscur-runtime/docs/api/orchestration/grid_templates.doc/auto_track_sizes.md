Converts a flat `grid-auto-columns` / `grid-auto-rows` [`GridTrack`] list into
the implicit-track [`TrackSize`] vector, reusing [`track_size_of`].

These lists carry no auto-repeat (the CSS layer rejects `auto-fill` / `auto-fit`
here), so every element maps 1:1; a stray auto-repeat would degrade to a
content-sized [`TrackSize::Auto`] via [`track_size_of`] rather than panic.

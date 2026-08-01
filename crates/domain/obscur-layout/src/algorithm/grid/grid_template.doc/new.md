Builds a template from explicit `columns` and `rows` track vectors (no
auto-repeat).

Pass an empty `rows` vector for a columns-only template whose rows are all
implicit (auto). This is the common no-repeat constructor; each vector
becomes a [`TrackList::from_tracks`].

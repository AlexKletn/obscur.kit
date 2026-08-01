The track template of a grid: its column and row sizing functions.

A `GridTemplate` is the CSS `grid-template-columns` / `grid-template-rows`
pair, each a [`TrackList`] (spec §4.1) — explicit tracks plus an optional
`repeat(auto-fill | auto-fit, …)` auto-repetition. Columns drive the horizontal
track sizing; rows drive the vertical. The auto-repeat (if any) is expanded to
a concrete track vector inside the layout algorithm, where the container's
content extent — the basis for the repetition count — is known.

# Implicit rows

`rows` may be **empty** (or shorter than the number of rows the items occupy).
Rows past the end of the explicit list are *implicit* and are sized as
[`TrackSize::Auto`] — as tall as the tallest item placed in them. An empty
`rows` list therefore yields an all-auto-row grid, which is the common
"columns only" template.

`columns` should be non-empty; the grid algorithm treats an empty column list
as a single flexible column so layout always has at least one column to place
items into.

# Implicit-track sizes (`grid-auto-columns` / `grid-auto-rows`)

[`auto_columns`](Self::auto_columns) / [`auto_rows`](Self::auto_rows) hold the
sizing functions for *implicit* tracks — those created past the explicit
template by placement / auto-flow. The list is **cycled**: the n-th implicit
track on an axis uses index `n % len`. An **empty** list means the default
single content-sized `auto` (byte-identical to the historical behaviour where
every implicit track was `Auto`).

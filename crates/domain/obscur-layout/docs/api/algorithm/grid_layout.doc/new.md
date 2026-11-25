Creates a uniform grid of `columns` equal-width columns and no gaps.

`columns` is clamped to a minimum of `1`, so a zero column count still
produces a valid single-column grid. Each column is a `1fr` track, so the
columns share the content width equally — the historical uniform-grid
behaviour — and every row is implicit (auto, content-sized).

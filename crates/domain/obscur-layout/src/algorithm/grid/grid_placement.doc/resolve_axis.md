Resolves one `start / end` axis pair into a concrete `(start, span)`.

`start` is `None` when the axis must be auto-flowed, `Some(i)` for a definite
0-based track index; `span` is always `≥ 1`.

# Negative (from-end) lines

A [`Line`](GridLine::Line) with a **negative** index counts back from the end
of the explicit grid: `-1` is the *last* line (1-based line `num_tracks + 1`),
`-2` the line before it, i.e. `Line(-k)` → 1-based line `num_tracks + 2 - k`.
Each negative edge is first normalised to that positive line against
`num_tracks`, then resolved by the same rules as a positive line — so
`1 / -1` spans the whole row and a bare `-1` lands on the last track. When any
edge was negative the resolved block is finally clamped to stay inside the
grid (`start ≥ 0`, `start + span ≤ num_tracks`); positive-only placements are
left byte-identical (never clamped, so definite positive lines past the
template still open implicit tracks).

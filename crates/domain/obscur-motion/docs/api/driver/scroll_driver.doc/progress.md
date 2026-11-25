Returns `((offset - start) / (end - start)).clamp(0.0, 1.0)` (spec §4.17).

A degenerate range (`start == end`) has no width and yields `0.0`.

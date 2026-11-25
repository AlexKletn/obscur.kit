Returns `((value - min) / (max - min)).clamp(0.0, 1.0)` (spec §4.17).

A degenerate domain (`min == max`) has no width and yields `0.0`; an
inverted domain normalizes correctly (see the type docs).

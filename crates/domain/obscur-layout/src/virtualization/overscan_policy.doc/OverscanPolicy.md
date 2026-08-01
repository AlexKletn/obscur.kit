How many extra items to materialize beyond the visible viewport, on each side
(spec §4.9 *overscan* — reduces blank flashes when scrolling).

A larger overscan trades more materialized work for fewer blank regions at the
leading/trailing edges during fast scrolls. Applied to a visible `[start, end)`
range via [`expand`](OverscanPolicy::expand), which saturates at `0` and the
item count so the result never leaves the collection's bounds.

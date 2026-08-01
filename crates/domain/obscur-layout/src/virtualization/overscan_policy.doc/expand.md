Applies this overscan to a visible half-open `[start, end)` range,
returning the expanded `(start, end)` (spec §4.9).

The start saturates at `0` and the end is clamped to `item_count`, so the
expanded range always stays within `[0, item_count]`.

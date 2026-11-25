Returns whether a transfer of `total_bytes` across `item_count` items is
admitted (spec §6.20).

A bounded axis rejects any value strictly greater than its cap;
unbounded axes admit everything. Both axes must pass.

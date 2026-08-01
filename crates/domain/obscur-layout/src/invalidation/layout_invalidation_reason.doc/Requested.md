A caller asked for this node's layout to be recomputed without naming a
finer cause.

The other variants each state *why* the geometry went stale, which lets a
future incremental pass decide how far to propagate. This one states only
that it did. It exists because a caller outside layout genuinely may not
know more — the runtime's `InvalidateLayout` operation is exactly that
statement — and mapping such a request onto one of the specific causes
would put a claim in its mouth that it never made.

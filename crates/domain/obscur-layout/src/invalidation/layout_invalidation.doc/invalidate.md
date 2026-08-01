Records that `node` must be recomputed for the given `reason`.

If `node` is already pending, the existing (first-recorded) reason is
kept; the new reason is discarded. This keeps merging deterministic and
order-stable, and never panics.

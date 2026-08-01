§11.5 — *Resolve Intrinsic Track Sizes*. Processes items in ascending span
batches (span-1 first). Within each batch, sub-step (a) grows intrinsic-min
bases and sub-step (d) grows intrinsic-max growth limits, each via
[`distribute_up_to_limits`] over only the item's spanned tracks, flushing
planned increases between sub-steps. Items are
`(start, span, contributions)`.

Each sub-step runs once per intrinsic [`Kind`], because the spec's sub-steps
are exactly that: "content-based minimums" grows the min-content tracks by the
min-content contribution, "max-content minimums" grows the max-content tracks
by the max-content one. A track has one min and one max sizing function, so
the passes touch disjoint sets.

Their order is not fully inert, though: besides the `sum` each later pass
measures its extra space against, `flush_limit` clears the §11.5.1
infinitely-growable flag that the next pass reads (see the body). For a grid
whose tracks are sized by one kind — the common case — that is unobservable.
For a mixed grid it is a real ordering dependency, and no spec-exactness is
claimed for one.

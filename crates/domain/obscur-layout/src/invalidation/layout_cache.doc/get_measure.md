Returns the cached measure result for `node` **iff** it was measured
under constraints that exactly equal `constraints`; otherwise `None`.

This is the early-exit for stable geometry: identical constraints reuse
the cached size, different constraints miss. A disabled cache always
returns `None`.

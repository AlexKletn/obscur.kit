Creates a store rooted at `root`, creating the directory (and any missing
parents) if it does not already exist.

A pre-existing directory is fine; only a genuine creation failure other than
"already exists" is surfaced — and even that is swallowed, since the store is
best-effort and a later `get`/`put` against an absent directory simply
misses.

Caches the parse of each CSS source, keyed by its identity **and** revision
(spec §4.12: "CSS parsing is cached per unique source/version and invalidated
explicitly").

# What decides a hit

The `(SourceId, SourceVersion)` pair, never the text. A source whose owner has
not bumped its version is the same source, and re-parsing it would be exactly
the work this exists to avoid — including re-reading the very text a hash would
have to walk. An [`ANONYMOUS`](SourceId::ANONYMOUS) source is deliberately
**not** cached: it has no stable identity, so two unrelated anonymous sheets
would collide under one key and the second would silently get the first's rules.

# Explicit invalidation

Two ways, both the owner's choice: bump the source's
[`version`](CssSource::version) (the entry no longer matches), or call
[`invalidate`](Self::invalidate)/[`clear`](Self::clear). Nothing here watches
files or times anything out — §4.12 asks for explicit invalidation, and a cache
that expires entries on its own would be neither explicit nor predictable.

A filesystem-backed [`CacheStore`] that persists each derived artifact as a
single file under a cache directory (spec §5.20, §2.7).

Each entry file is named from its [`CacheKey`] (`<resource>-<variant>.oc`) and
stores the artifact's [`ResourceVersion`] stamp inline ahead of the bytes, so a
[`CacheStore::get`] served by a *different* process or run still enforces the
§5.20 compatibility check. Because every entry is derived data, the entire
directory is disposable: [`CacheStore::clear`], [`CacheStore::remove`], or
deleting the directory outright is always safe.

Writes are best-effort. `put` swaps a fully written temp file into place with a
rename, so a crash never leaves a half-written entry, and any I/O failure during
`put` is swallowed — a failed write simply means a later `get` misses and the
artifact is recomputed.

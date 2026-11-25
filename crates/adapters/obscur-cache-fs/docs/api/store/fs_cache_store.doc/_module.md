A filesystem-backed [`CacheStore`] adapter (spec §5.20 persistent derived
cache, §2.7 "persistent cache adapter").

Spec §5.20 describes a store of expensive *derived* artifacts (compiled
shaders, rasterised glyphs, prepared display lists) that survives across runs:
it is **versioned**, **content-addressed where useful**, **compatibility-aware**
(a stored entry is served only while its version still matches), and
**disposable** — every entry is derived data that can be safely deleted and
recomputed. The kit's core performs no filesystem access itself; an interface
adapter does (spec §2.7). This module is that adapter: [`FsCacheStore`]
implements the pure [`CacheStore`] port using only `std`, persisting each entry
as a single file under a cache directory.

# On-disk format

One file per entry. The filename is content-addressed from the key so distinct
keys never collide:

```text
<resource-id>-<variant>.oc
```

where `<resource-id>` is [`CacheKey::resource`]'s raw `u64` and `<variant>` is
[`CacheKey::variant`]. The file contents are the 8-byte little-endian version
stamp followed by the raw artifact bytes:

```text
[ version_stamp: u64 LE (8 bytes) ][ artifact bytes … ]
```

[`CacheStore::get`] reads the 8-byte header and returns the payload only when it
equals the requested version (§5.20 compatibility-awareness); a stale header, a
truncated file, or any I/O error is treated as a miss.

Regular files named `<resource-id>-<variant>.oc` are disposable cache entries.
The adapter leaves other files in the directory untouched.

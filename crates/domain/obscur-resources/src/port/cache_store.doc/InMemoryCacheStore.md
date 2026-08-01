A volatile in-memory reference [`CacheStore`] (spec §5.20).

`InMemoryCacheStore` keeps entries in a [`HashMap`] keyed by [`CacheKey`],
storing each artifact's [`ResourceVersion`] alongside its bytes so
[`CacheStore::get`] can enforce the §5.20 compatibility check. It is a
reference implementation for tests and headless use: because it holds
everything in memory, nothing survives across runs — a real *persistent*
derived cache lives in an adapter. Its entries are derived data, so clearing
or dropping it is always safe.

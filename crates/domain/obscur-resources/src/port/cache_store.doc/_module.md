The hexagonal port for the persistent derived-artifact cache (spec §5.20).

This module defines [`CacheStore`], the port for the §5.20 persistent derived
cache: a store of derived artifacts (compiled shaders, rasterised glyphs,
prepared display lists) that survives across runs. Per spec §5.20 the store is
**versioned** and **compatibility-aware** (a stored entry is only served when
its version still matches), **content-addressed where useful** (via the
[`CacheKey`] variant), and **disposable** — every entry is derived data that
can be safely deleted and recomputed. Adapters back it with a real on-disk
store; the colocated [`InMemoryCacheStore`] is a volatile reference
implementation.

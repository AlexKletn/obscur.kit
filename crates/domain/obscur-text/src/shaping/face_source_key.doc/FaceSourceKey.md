The stable identity of a [`FontSource`] used inside a cache key.

System and bundled sources are keyed by their name (tagged so a `System`
name never collides with an identical `Bundled` name). In-memory font bytes
are keyed by the backing allocation's pointer and length — a cheap, stable
identity — instead of hashing the whole byte buffer on every lookup. Two
`Memory` sources that share the same `Arc<[u8]>` (the common case: one face
shaped repeatedly) therefore hit the cache, and distinct allocations never
collide even if their lengths match.

Shared by every shaping cache in this module (the per-run
[`CachingShaper`](crate::shaping::CachingShaper) and the paragraph cache) so
they compute face identity identically.

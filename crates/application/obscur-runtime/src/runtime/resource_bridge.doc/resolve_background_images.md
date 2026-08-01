Resolves every not-yet-cached `background-image: url(...)` referenced by the
freshly `computed` styles into a decoded [`SceneImage`] in
[`image_cache`](Self::image_cache) (spec §5.15 load → §5.16 decode).

For each new URL this drives the full hub lifecycle: `request` a handle for
a [`ResourceKind::Image`] source built from the URL, `begin_load`, then
`loader.load` → `decoder.decode` → build a `SceneImage` from the decoded
RGBA8 + intrinsic size, and `complete` the handle. A loader/decoder error
(or a malformed decoded buffer) `fail`s the handle and caches nothing, so
the image is simply not painted — no panic. With no loader/decoder installed
this is a no-op (the cache stays empty). Synchronous with the filesystem
loader (ready the same frame); the design is async-friendly — an unready URL
is just absent from the cache and paints on a later frame once decoded.

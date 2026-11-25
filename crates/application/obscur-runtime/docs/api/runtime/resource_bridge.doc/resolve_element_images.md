Resolves every not-yet-cached element image `src` (an `<img>` /
[`Image`](obscur_primitives::Image) primitive's source) referenced by the
retained tree into a decoded [`SceneImage`] in
[`image_cache`](Self::image_cache) (spec §5.15 load → §5.16 decode).

Element images share the **same** cache, hub lifecycle, and load/decode path
as `background-image: url(...)` (see
[`resolve_background_images`](Self::resolve_background_images)), keyed by the
raw `src` string. A URL already resolved (or already attempted and failed) is
not retried; a loader/decoder error `fail`s the handle and caches nothing, so
the image is simply not painted — no panic. With no loader/decoder installed
this is a no-op (the honest-capability default).

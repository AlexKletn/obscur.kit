An in-memory map of converted [`BezPath`]s keyed by a content hash.

Path conversion (obscur-scene → kurbo) is pure but not free; a scene that
re-draws the same geometry every frame would otherwise re-run the conversion
each time. This cache stores the converted [`BezPath`] under a caller-computed
`u64` content hash so a repeat draw is a hash-map lookup. It holds no GPU
resources and is fully testable without a device.

The key is opaque: callers are responsible for hashing the source path's
geometry into a stable `u64`. Two different paths that collide on a hash would
alias — callers must use a hash wide enough to make that negligible.

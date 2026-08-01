Cached owned wgpu gaussian-blur pipeline set for `filter: blur()` (spec
§5.2). Built lazily on the first blurred frame (it needs the device) and
reused across frames; a scene with no blur layer never constructs it, so
the non-blur path is byte-identical to before.

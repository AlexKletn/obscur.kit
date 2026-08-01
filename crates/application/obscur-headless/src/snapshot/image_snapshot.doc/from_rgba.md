Captures an image snapshot from raw pixel `bytes` of a `width` × `height`
raster.

Any 4-bytes-per-pixel layout works — RGBA8, BGRA8, premultiplied or not —
as long as both sides of a comparison use the same one, since a
per-channel delta is computed bytewise.

The `bytes` are hashed with FNV-1a into a stable content digest and
retained for [`compare_within`](Self::compare_within): identical
dimensions and bytes always produce an equal snapshot.

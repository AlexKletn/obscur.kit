Identifies one cached artifact derived from a resource (spec §5.19 runtime
caches, §5.20 persistent derived cache).

A single resource can back several cached derivatives — a display list at one
scale, a rasterised mask at another size, a compiled shader for a given format
— so a key pairs the owning [`ResourceId`] with a `variant` discriminator. The
`variant` is an opaque `u64` the caller derives by hashing the derivation
parameters (target size, scale, pixel format, …) or the artifact's content
(spec §5.20 *content-addressed where useful*). Variant `0` denotes the
canonical, unparameterised artifact.

This is pure bookkeeping data: it stores no bytes and performs no hashing
itself, so the caller controls how a `variant` is computed. The full derive
set (including `Hash`, `Eq`, and `Ord`) lets a key serve directly as a
`HashMap` or `BTreeMap` index.

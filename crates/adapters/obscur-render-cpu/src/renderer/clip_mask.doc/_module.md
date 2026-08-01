The per-pixel coverage mask a [`Clip`] resolves to on the software backend
(spec §5.2 "clipping must match across backends", §5.8 mask fallback).

`tiny-skia` has no clip stack: a clip is a [`tiny_skia::Mask`] passed to every
draw call, and pushing one means intersecting the new outline into the enclosing
mask. That makes clip resolution a small algorithm with a vocabulary of its own —
parent coverage, effective coverage, "admits nothing" versus "states no extent" —
and one rule that must not be split in two: a clip command and a layer's own clip
naming the same region have to confine drawing identically, because Vello
resolves both through one rule and §5.2 requires the two backends to agree. Both
callers therefore enter through [`resolve_clip_entry`].

The interface is two calls; the internals are the shape lowering, the empty-mask
answer, and the deliberate asymmetry between an empty outline (which admits
nothing) and a mask clip (which states no extent and so inherits).

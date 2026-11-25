Which content contribution an intrinsic sizing function is fed by (§11.5).

One kind per intrinsic track type, and the reason this is an enum rather than
a bool: a track sized by `min-content` and one sized by `max-content` are both
"intrinsic", but they must be grown by *different numbers*, so the item has to
supply — and each sub-step has to pick — the right one.

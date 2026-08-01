Resolves the drawn image tile width/height (logical px) from `object-fit`, the
image's intrinsic size `(iw, ih)`, and the box size `(bw, bh)` — the replaced-
element analog of
[`resolve_background_tile_size`](super::background_paint::resolve_background_tile_size).

* `fill` → the box size (stretch, aspect ignored — the CSS initial);
* `contain` → the largest aspect-preserving size that fits (min scale);
* `cover` → the smallest aspect-preserving size that covers (max scale);
* `none` → the intrinsic size (unscaled);
* `scale-down` → `min(none, contain)` (intrinsic, shrunk only if it overflows).

A degenerate intrinsic size (`iw`/`ih` ≤ 0) cannot preserve an aspect ratio, so
it falls back to a full-box stretch (matching the background path).

Resolves the drawn tile width/height (logical px) from `background-size`, the
image's intrinsic size `(iw, ih)`, and the box size `(bw, bh)`.

* `auto` → intrinsic pixel size; `cover` → the smallest size that fully covers
  the box (max scale, aspect preserved); `contain` → the largest that fits
  (min scale, aspect preserved); explicit → each axis resolved against the box,
  a single `auto` axis derived from the other to preserve the intrinsic aspect
  ratio (both `auto` = intrinsic). A degenerate intrinsic size falls back to a
  full-box stretch.

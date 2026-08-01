The ambient context in which an intrinsic measurement is performed.

Passed to every [`IntrinsicMeasure`](super::IntrinsicMeasure) call so a
provider can size content correctly for the current display and viewport.

For the Phase-1 headless slice this carries only:
* `scale_factor` — the device pixel ratio, for DPI-aware measuring;
* `viewport` — the viewport size, used later for viewport-relative resolution
  (e.g. `vw`/`vh` units).

This type is intentionally minimal and **will grow** in later phases (font
context, locale/BiDi, writing mode). It is a plain value: `Copy` and cheap
to pass by reference.

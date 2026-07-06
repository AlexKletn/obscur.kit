# Visual / paint properties — remaining items

The visual paint layer is now substantial and verified end-to-end (see
`crates/facade/obscur-kit/tests/visual_css.rs`). Implemented: `background-color`,
`background-image: {linear,radial,conic}-gradient(...)`,
`border-{side}-{width,style,color}`
(painted), `border-radius` (rounded fill + clip), `box-shadow` (drop + inset),
`filter: drop-shadow(...)` (via the box-shadow `DrawShadow` path),
2D `transform` (translate/scale/rotate/skew/matrix + transform-origin), plus a
named-color table. Rendering is real on the Vello backend and headless-
approximated on the CPU (tiny-skia) backend.

The following are deferred. Each is documented at its site in code.

## Gradients & backgrounds
- **`background-image: url(...)`** — no image backgrounds. The decoder adapter
  (`obscur-resource-decode`, image/svg/font) exists; wiring a decoded image into
  a `Brush`/paint + the ResourceHub load is the work.
- **Multiple background layers** — a comma list of backgrounds; only a single
  gradient/color is honored.
- **Non-percentage (length) gradient stop positions** — stops take `%` only
  (applies to linear / radial / conic alike).
- **Radial/conic gradient fidelity** — `radial-gradient` and `conic-gradient` are
  lowered and reach `Brush::Radial` / `Brush::Conic` (real on Vello, first-stop
  flatten on CPU). Accepted subset: radial `[circle|ellipse]? [<extent-keyword> |
  <length|%> radius]? [at <position>]?`; conic `[from <angle>]? [at <position>]?`.
  An `ellipse` shape is collapsed to a single radius (the scene models a
  concentric circle); a `%` radius is approximated against the box's smaller
  half-extent; `at <position>` accepts the same keyword/`px`/`%` grammar as
  `transform-origin`. Angular color hints and per-stop `<angle>` positions
  (conic), and interpolation color spaces, are unsupported.
- **Corner gradient directions** — `to top left` etc. use the 45° diagonal, not
  CSS's aspect-ratio-aware corner angle.

## Border
- **`border-style` variants** — `dashed`/`dotted`/`double`/`groove`/… all render
  as `solid`; only `none`/`solid` are distinct.
- **Non-uniform rounded borders** — a border with a radius and unequal side
  widths/colors is approximate (edges clipped to the rounded box, no per-corner
  mitre geometry).
- **Elliptical `border-radius`** — the `<h> / <v>` two-radius-per-corner syntax is
  unsupported; a single radius per corner only.

## Shadow
- **CPU gaussian blur** — the CPU backend paints a flat reduced-alpha offset rect
  (no blur); Vello does real `draw_blurred_rounded_rect`.
- **Inset shadow fidelity (Vello)** — inset shadows are not clipped to the box
  interior (approximate).
- **Shadow on a rounded element** — a drop shadow sits inside the element's own
  subtree clip (documented approximation of the paint order).
- **`filter: drop-shadow(...)`** — DONE. Lowered to a drop `BoxShadow` (spread `0`,
  not inset) and painted BEHIND the element via the same `DrawShadow` path as
  `box-shadow` (Vello real gaussian, CPU flat offset approx), emitted after the
  box-shadow drop shadows. The shadow follows the element's border BOX, not its
  content alpha (a documented approximation — real CSS drop-shadow follows the
  rendered content's alpha, including its `box-shadow`).

## Transform
- **Percentage `translate`** — `translate(50%)` is treated as `0` (the box size
  is not known at CSS-lower time); resolve against the box in the runtime.
- **Transformed hit-testing** — the pointer/hit-test path is NOT transformed, so
  a transformed element's interactive region is still its untransformed box.
- **3D transforms** — `translate3d`/`perspective`/`rotateX`… unsupported (2D only).

## Done
- ~~**`opacity` as a composited layer**~~ — DONE. `opacity < 1` now brackets the
  element's whole subtree with `PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`
  in the scene coordinator (nested transform → opacity layer → rounded clip), and
  is honored end-to-end: Vello via `scene.push_layer(alpha)`, and the CPU backend
  via a real offscreen-pixmap group blend (overlapping opaque children composite as
  one group, nested layers multiply). The layer's own clip / blend-mode / transform
  / effects are still approximated on the CPU backend (only opacity is honored).

## Not started
- **`filter` / `backdrop-filter`** — the general property is BLOCKED on Vello 0.9,
  which has no content-blur or color-matrix primitive: `blur()`, `brightness()`,
  `contrast()`, `grayscale()`, `backdrop-filter`, … cannot be expressed and are
  **silently dropped** at CSS lower time (not faked). Only the single
  GPU-expressible function, `filter: drop-shadow(...)`, is implemented (see the
  Shadow section above); `supports_effect("filter" / "backdrop-filter")` reports
  `Unavailable` on both backends, `"drop-shadow"` reports `Emulated`.
- **`outline` / `outline-offset`** — an outline that does not affect layout.
- **`clip-path`** — arbitrary clip shapes (the scene has path clips).
- **`mix-blend-mode` / `background-blend-mode`** — the scene has blend on layers.
- **`visibility: hidden` vs `display: none`** paint suppression — confirm
  `visibility` is honored at paint time.

## Paint-time units
- Only absolute `px` lengths are honored at PAINT time (border width, radius,
  shadow offsets/blur/spread) — relative/`%` units inset via layout but do not
  paint, mirroring the earlier `font-size` limitation. Resolve against the box in
  the coordinator.

---

Ranked by likely value: `background-image: url()` > transformed hit-testing >
`outline` > the rest.

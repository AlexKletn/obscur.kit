# Visual / paint properties — remaining items

The visual paint layer is now substantial and verified end-to-end (see
`crates/facade/obscur-kit/tests/visual_css.rs`). Implemented: `background-color`,
`background-image: linear-gradient(...)`, `border-{side}-{width,style,color}`
(painted), `border-radius` (rounded fill + clip), `box-shadow` (drop + inset),
2D `transform` (translate/scale/rotate/skew/matrix + transform-origin), plus a
named-color table. Rendering is real on the Vello backend and headless-
approximated on the CPU (tiny-skia) backend.

The following are deferred. Each is documented at its site in code.

## Gradients & backgrounds
- **radial-gradient / conic-gradient** — only `linear-gradient` is lowered; the
  scene `Brush` already models radial/conic, so this is a CSS-lowering + endpoint
  task.
- **`background-image: url(...)`** — no image backgrounds. The decoder adapter
  (`obscur-resource-decode`, image/svg/font) exists; wiring a decoded image into
  a `Brush`/paint + the ResourceHub load is the work.
- **Multiple background layers** — a comma list of backgrounds; only a single
  gradient/color is honored.
- **Non-percentage (length) gradient stop positions** — stops take `%` only.
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

## Transform
- **Percentage `translate`** — `translate(50%)` is treated as `0` (the box size
  is not known at CSS-lower time); resolve against the box in the runtime.
- **Transformed hit-testing** — the pointer/hit-test path is NOT transformed, so
  a transformed element's interactive region is still its untransformed box.
- **3D transforms** — `translate3d`/`perspective`/`rotateX`… unsupported (2D only).

## Not started
- **`opacity` as a composited layer** — `opacity` is a computed property but is
  not applied as a group opacity layer (`PushLayer`); a partially-transparent
  subtree is not composited. The scene has `PushLayer(SceneLayer)` for this.
- **`filter` / `backdrop-filter`** (blur, brightness, drop-shadow, …).
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

Ranked by likely value: `opacity` layer > `background-image: url()` >
transformed hit-testing > radial-gradient > `outline` > the rest.

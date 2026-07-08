# Visual / paint properties — remaining items

The visual paint layer is now substantial and verified end-to-end (see
`crates/facade/obscur-kit/tests/visual_css.rs`). Implemented: `background-color`,
`background-image: {linear,radial,conic}-gradient(...)`,
`background-image: url(...)` (load + decode via the ResourceHub into a
`DrawImage`, placed + sized + tiled),
`background-{size,position,repeat}` (tiled + placed `DrawImage`),
`border-{side}-{width,style,color}`
(painted), `border-radius` (rounded fill + clip), `box-shadow` (drop + inset),
`filter: drop-shadow(...)` (via the box-shadow `DrawShadow` path),
2D `transform` (translate/scale/rotate/skew/matrix + transform-origin), plus a
named-color table. Rendering is real on the Vello backend and headless-
approximated on the CPU (tiny-skia) backend.

The following are deferred. Each is documented at its site in code.

## Gradients & backgrounds
- ~~**`background-image: url(...)`**~~ — DONE. A `url()` background loads (via the
  injected `ResourceLoader`) + decodes (via the injected `ResourceDecoder`)
  through the runtime-owned `ResourceHub` into a cached `SceneImage`, which the
  scene coordinator paints as a `DrawImage` over the node's box (above
  `background-color`, below the border/content). The `DrawImage` now carries a
  placement model `{ image, tile, region, repeat_x, repeat_y }` (see below).
  Remaining: multiple background layers, and the fs resolve is synchronous (an
  unready URL simply paints a later frame).
- ~~**`background-size` / `-position` / `-repeat`**~~ — DONE. Three new NOT-inherited
  computed properties (`BackgroundSize` = `auto` / `cover` / `contain` / explicit
  per-axis `<length|%|auto>`; `BackgroundPosition` = per-axis `<length|%|keyword>`,
  CSS position formula; `BackgroundRepeat` = `repeat` / `no-repeat` / `repeat-x` /
  `repeat-y`), fully registered (id + computed registry + coverage guard + name
  table + initial + value/computed accessors). The scene coordinator resolves the
  intrinsic image size + box into the `DrawImage` `tile` (position + size) and
  `repeat_x`/`repeat_y` flags; the `region` (border box) clips the image(s). Both
  backends honor it — the CPU backend loops copies + clips to `region`, and the
  Vello backend uses the same bounded per-copy loop inside a pushed `region` clip
  layer (peniko 0.6 `x_extend`/`y_extend` exist but Vello 0.9's `draw_image` draws a
  single instance, so extend is not used for tiling). The CSS **initial** background
  is the CSS-correct intrinsic-size tile at `0% 0%` repeated on both axes — NOT a
  full-box stretch (a plain full-box stretch is `tile == region` + no repeat).
  Deferred: `space` / `round` fold to `repeat` (no gap-distribution / integer-fit);
  multiple background layers; sub-pixel tiling can show a hairline seam at some
  scales; `em`/`vw`/… size/position units fall back (px + % are exact).
- ~~**Multiple background layers**~~ — DONE. `PropertyValue::BackgroundLayers`
  (comma list of `{Linear|Radial|Conic|Url}` layers) painted back-to-front; a
  single layer is byte-identical to before.
- ~~**`repeating-linear-gradient`**~~ — DONE. A repeating flag + period on the
  linear gradient; the coordinator sizes the axis to one period and tiles via
  `ExtendMode::Repeat` (Vello tiles; CPU flattens to the first stop). Combined
  with multiple layers this expresses the graph-paper grid. (`repeating-radial`
  is still deferred.)
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
- ~~**`border-style: dashed`/`dotted`**~~ — DONE. Scene `Stroke` gained a
  `dash`/`dash_offset`; a uniform dashed/dotted border strokes the (rounded) box
  outline on both backends (Vello kurbo dashes, CPU tiny-skia `StrokeDash`).
  Non-uniform dashed still falls back. `double`/`groove`/`ridge`/`inset`/`outset`
  still render as `solid`.
- ~~**Mitered border triangles**~~ — DONE. Solid non-rounded borders render as
  mitered trapezoid quads, so the `width:0;height:0;border-…` CSS-triangle trick
  works. (Known follow-up: vertical edges arrive flipped through the
  computed-style path for degenerate 0-height boxes — see the audit's residue.)
- **Non-uniform rounded borders** — a border with a radius and unequal side
  widths/colors is approximate (edges clipped to the rounded box, no per-corner
  mitre geometry).
- **Elliptical `border-radius`** — the `<h> / <v>` two-radius-per-corner syntax is
  unsupported; a single radius per corner only.

## Shadow
- **CPU gaussian blur** — the CPU backend paints a flat reduced-alpha offset rect
  (no blur); Vello does real `draw_blurred_rounded_rect`.
- ~~**Inset shadow fidelity (Vello)**~~ — DONE. Vello now clips+isolates to the
  box and punches the lit interior via `Compose::DestOut` (soft inner frame);
  CPU draws an even-odd rounded-rect ring. (Drop shadows byte-identical.)
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
- **`filter` / `backdrop-filter` — color-matrix subset only** — `filter: blur()`
  and `backdrop-filter` (acrylic) ARE now implemented on the Vello backend via our
  own gaussian + composite **wgpu passes on offscreen textures** over the scene
  (this is what the Fluent theme's acrylic uses); `filter: drop-shadow()` is
  implemented via the `DrawShadow` path (see Shadow). What remains BLOCKED on
  Vello 0.9 is the **color-matrix** family — `brightness()`, `contrast()`,
  `saturate()`, `grayscale()`, `invert()`, `sepia()`, `hue-rotate()` — which has
  no Vello primitive and is **silently dropped** at CSS lower time (not faked).
  `supports_effect` reports the color-matrix subset as `Unavailable`.
- ~~**`outline` / `outline-offset`**~~ — DONE. Painted as one stroked `DrawPath` at
  the border box inflated by `offset + width/2` (rounded if a radius, dashed via
  the dash stroke); reserves no layout space; emitted outside the node's clip.
- ~~**`overflow` clipping**~~ — DONE. `overflow: hidden`/`clip`/`scroll`/`auto`
  bracket the subtree with a `PushClip` to the box; `scroll`/`auto` clip but paint
  no scrollbars (no scroll UI / gesture wiring yet).
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

Ranked by likely value: transformed hit-testing > `outline` >
`background-size`/`repeat`/`position` for `url()` images > the rest.

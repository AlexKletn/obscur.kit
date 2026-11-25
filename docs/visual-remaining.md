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
  An unready URL paints on a later frame once its resource is available.
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
  Raster, SVG and gradient layers resolve size, position and repeat by
  cycling each authored list. `space` distributes gaps between whole tiles
  and `round` resizes tiles to fit a whole number.
- ~~**Multiple background layers**~~ — DONE. `PropertyValue::BackgroundLayers`
  holds linear, radial, conic, URL and empty layers in authored order. The
  coordinator paints them back-to-front over the base color.
- **`background-clip`** clips each background layer to its selected border,
  padding or content box. The last image layer selects the clip for the base
  color. `background-origin` selects the positioning box separately and
  defaults to the padding box. Both box choices can be set in `background`.
  `background-attachment` supports scroll, local and fixed. Local uses the
  scrollable extent for placement and clips the requested box against the
  stationary scrollport.
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
- **Gaussian shadows**. CPU and Vello blur outset and inset shadows. For an
  inset shadow both use a blurred hole inside the rounded box.
- **Shadow on a rounded element** — a drop shadow sits inside the element's own
  subtree clip (documented approximation of the paint order).
- **`filter: drop-shadow(...)`** — DONE. Lowered to a drop `BoxShadow` (spread `0`,
  not inset) and painted BEHIND the element via the same `DrawShadow` path as
  `box-shadow` (Gaussian on CPU and Vello), emitted after the
  box-shadow drop shadows. The shadow follows the element's border BOX, not its
  content alpha (a documented approximation — real CSS drop-shadow follows the
  rendered content's alpha, including its `box-shadow`).

## Transform
Percentage translations resolve against the current border box. Hit testing
uses the inverse accumulated transform.

- **3D transforms** — `translate3d`/`perspective`/`rotateX`… unsupported (2D only).

## Done
- ~~**`opacity` as a composited layer**~~ — DONE. `opacity < 1` now brackets the
  element's whole subtree with `PushLayer(SceneLayer::with_opacity(a))` … `PopLayer`
  in the scene coordinator (nested transform → opacity layer → rounded clip), and
  is honored end-to-end: Vello via `scene.push_layer(alpha)`, and the CPU backend
  via a real offscreen-pixmap group blend (overlapping opaque children composite as
  one group, nested layers multiply). CPU also honors layer clips and
  separable blend modes.

- **`visibility: hidden`** preserves layout and suppresses the node's own
  paint and pointer targeting. Descendants inherit visibility and can override
  it with `visibility: visible`. Hidden nodes are excluded from focus and the
  accessibility tree. The `collapse` keyword remains open.

## Not started
- **CPU filter coverage**. Vello's GPU compositor supports blur, backdrop
  blur, and the supported CSS color-matrix functions, including brightness,
  contrast, saturation, grayscale, invert, sepia and hue rotation. CPU layers
  apply the folded CSS color matrix, Gaussian layer blur and backdrop blur.
  CPU Gaussian blur, including shadows, is limited to a sigma of 16 device
  pixels.
- ~~**`outline` / `outline-offset`**~~ — DONE. Painted as one stroked `DrawPath` at
  the border box inflated by `offset + width/2` (rounded if a radius, dashed via
  the dash stroke). It reserves no layout space, escapes the node's overflow
  clip and remains subject to its CSS `clip-path`.
- **`overflow` clipping and scrolling** are implemented. Scroll containers
  can paint scrollbar tracks and thumbs.
- **CSS `clip-path: polygon(...)`** clips painting and hit testing. Polygon
  vertices use the selected border, padding or content box. Each box can also
  serve as a rounded clip on its own. For kit layout boxes, `fill-box` resolves
  to the content box, and `stroke-box` and `view-box` resolve to the border box.
  `circle()` and `ellipse()` support
  explicit radii, side distances and an authored center. `inset()`, `rect()`
  and `xywh()` support elliptical corner radii. `rect()` uses edge coordinates
  with `auto` defaults, and `xywh()` uses an origin and size. `path()` accepts
  SVG path data with either fill rule. Math functions in polygon vertices,
  rectangle coordinates and corner radii, and circle/ellipse radii and centers
  resolve against the applicable geometry during painting and hit testing.
  Curved hit regions use the flattened path outline.
- **Image masks (§5.8)**. The scene carries an image, bounds and coverage mode.
  CPU, classic Vello and Lyon apply alpha and luminance image masks to isolated
  groups. Hybrid renders those frames through classic Vello and reports the
  fallback. A mask declaration without a coverage source always needs a
  fallback.
- **`background-blend-mode`** cycles the authored blend modes over the image
  layers. The background color and images form an isolated group. Both CSS
  blend properties accept the nonseparable modes `hue`, `saturation`,
  `color` and `luminosity`.

## Paint-time units
- Only absolute `px` lengths are honored at PAINT time (border width, radius,
  shadow offsets/blur/spread) — relative/`%` units inset via layout but do not
  paint, mirroring the earlier `font-size` limitation. Resolve against the box in
  the coordinator.

---

## CPU and Vello verification

The `fractional_edge`, `svg_vector` and `text_cross_backend` facade suites
were run with default options and again with `-- --ignored --test-threads=1`.
All 13 default cases and all 10 GPU cases passed. This covers fractional-scale
edges, gradient sampling, premultiplied alpha, SVG clipping and text pixels.
Filter and mask parity are outside these suites.

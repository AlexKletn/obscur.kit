The **capabilities** scene, authored once and rendered by three drivers:
the headless CPU→PNG example (`capabilities_png.rs`), the headless Vello→PNG
example (`capabilities_vello_png.rs`), and the live windowed Vello example
(`capabilities_window.rs`).

It showcases the CSS features ported in the "math campaign", authored purely
through the toolkit's **CSS string + `View` tree** surfaces (`obscur_kit::*`)
and driven through the real `obscur-css` frontend + runtime coordinator +
renderer — no hand-built paint commands, no backend types. The same view +
stylesheet therefore feed the software rasterizer and the GPU surface
unchanged.

# The fourteen section cards

1. **Color spaces** — swatches filled with `oklch()`, `lab()`, `hsl()`,
   `oklab()` (CSS Color 4 parsing → sRGB). CPU-visible.
2. **Oklab gradient** — a `linear-gradient(90deg, #f00, #0f0)` bar,
   interpolated in Oklab (Vello only; the CPU backend flattens it to the first
   stop, so the CPU card is captioned "GPU / Vello").
3. **linearRGB blur** — a bright box under `filter: blur()`, convolved in
   linear light (Vello only; the CPU card is captioned "GPU / Vello").
4. **border-radius overlap clamp** — a wide-short box whose `border-radius`
   would overflow, uniformly scaled down so it never does.
5. **box-shadow spread** — a rounded box whose shadow radii grow with spread.
6. **Stacking order** — a `position: relative` box earlier in the tree paints
   ON TOP of a later non-positioned sibling (§9.9 paint layers).
7. **Flex grow/shrink** — a `display: flex` row distributing free space.
8. **Grid two-value tracks** — `grid-template-columns: minmax(...) 1fr 1fr`.
9. **Margin collapsing** — adjacent block margins collapse to the max.
10. **Text spacing + transform** — `letter-spacing` / `word-spacing` /
    `text-transform: uppercase`.
11. **white-space** — the same source text under `normal` / `pre` / `nowrap`.
12. **Ellipsis** — `text-overflow: ellipsis` on overflowing `nowrap` text.
13. **Transform interpolation** — a square at the interpolated `rotate(45°)`,
    full size (the window driver animates it via `MotionValue::Transform`).
14. **CPU vs GPU** — an honesty legend: which cards are GPU-only.

The gradient (§2) and blur (§3) are GPU-class: on the CPU backend they degrade
to a documented approximation (a gradient paints its first stop; a blur is a
coarse approximation) per the project's CPU/GPU effect-split policy — so their
CPU cards carry a visible "GPU / Vello" caption rather than a misleading fake.
Everything else (layout, color parsing, radius/spread geometry, stacking,
flex/grid, margin collapsing, text, and the transform) renders faithfully on
both backends.

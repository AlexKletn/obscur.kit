The complete **Minimal / Neutral** stylesheet (theme index 0) — an airy, light
"Helvetica" system: warm off-white page (`#f6f6f3`), white cards with a single
hairline border (`1px #e7e7e1`) and *no* shadow, a small 8px radius, one calm
muted-blue accent (`#407cc5`, the spec's `oklch(0.58 0.13 255)` converted to
sRGB), and lots of whitespace. Authored against the same semantic classes as
[`MATERIAL_CSS`](super::material_theme::MATERIAL_CSS); the layout numbers (card widths, row heights, control sizes)
are shared so the fixed view fits. Restraint is the whole aesthetic: flat solid
fills only, so this reproduces faithfully on the CPU backend.

# Approximated (our engine's known gaps)
- **`letter-spacing` / `text-transform`** on the mono eyebrow labels are dropped
  (labels are already uppercased in the view); no engine support.
- The card-cover **`repeating-linear-gradient` hatch** is approximated by a flat
  light fill (`background-color`), matching the CPU effect-split policy.

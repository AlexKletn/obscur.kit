The **Skeuomorphic** stylesheet (theme index 4) — the gradient/shadow-heavy
"glossy plastic + brushed beige" look, authored against the same semantic
classes as [`MATERIAL_CSS`](super::material_theme::MATERIAL_CSS). The view is untouched; only these tokens change.

# Techniques (the ones that make it skeuomorphic)

- **2-stop vertical `linear-gradient` faces.** Every panel/button gets a
  top-light → bottom-dark ramp (`linear-gradient(#light,#dark)`), which on the
  Vello backend paints a real smooth ramp — the source of the "molded plastic"
  sheen. On the CPU backend the same gradient degrades to its FIRST stop (a
  flat light fill) per the effect-split policy, so the CPU PNG looks flat by
  design — that contrast is the whole point of the two renders.
- **`inset` gloss highlight + `outset` drop shadow in ONE `box-shadow` list.**
  The engine honors a comma list of shadows and paints drop shadows BEHIND the
  background and inset shadows ON TOP of it, so a single declaration like
  `box-shadow: inset 0 1px 0 rgba(255,255,255,.8), 0 2px 5px rgba(0,0,0,.25)`
  gives both the raised top-edge highlight and the cast shadow. Both blur
  through Vello's real gaussian; the CPU backend approximates them as flat
  offset rects.
- **1px bevel borders** (`1px solid #a9a496`) around the beige faces, and
  **recessed wells** for inputs / slider tracks (an `inset` shadow with no
  drop shadow reads as "sunken").
- **Radius ~10–12px** throughout for the rounded-hardware feel.

# Approximated / omitted (documented per project honesty policy)

- **`text-shadow` emboss** — the imported design embosses every label with
  `text-shadow: 0 1px 0 rgba(255,255,255,.x)`. We have no `text-shadow`
  property, so it is **omitted**; text renders crisp without the engraved
  highlight. This is the one signature we cannot reproduce.
- **>2-stop / `repeating-linear-gradient` gloss** (a hard specular band across
  the top third) is approximated by a plain 2-stop ramp — the engine models a
  single `linear-gradient` with auto-distributed stops, not a repeating one.
- **Per-corner / multi-layer inset bevels** collapse to one `inset` shadow with
  uniform corner radii (`draw_shadow` blurs a uniformly-rounded rect).

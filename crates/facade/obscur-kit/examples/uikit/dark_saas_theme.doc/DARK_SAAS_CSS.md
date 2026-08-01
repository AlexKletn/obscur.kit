The complete **Dark SaaS** stylesheet (theme index 2) — a deep-slate dashboard:
near-black page (`#0c0e14`), near-black raised surfaces (`#14161f`) with subtle
`1px #232634` light-alpha borders, a vivid violet accent (`#7c5cff`, the spec's
`oklch(0.66 0.19 280)` / its `rgba(124,92,255)` glow converted to sRGB), light
text on dark (`#e6e8ee`) with muted greys (`#8b90a0`, `#6b7180`), and a small
10-14px radius. Inputs are recessed wells (`#0e1017`) a shade darker than the
cards. Same semantic classes and shared layout numbers as [`MATERIAL_CSS`](super::material_theme::MATERIAL_CSS); solid
/ rgba fills only, so it reproduces faithfully on the CPU backend.

# Approximated (our engine's known gaps)
- Mono eyebrow **`letter-spacing` / `text-transform`** dropped (no engine
  support; labels are pre-uppercased in the view).
- The title's **text-gradient** (`#fff → #b9b4ff`) and the accent **glow**
  `box-shadow`s are GPU-class; text keeps a solid light fill and glows degrade to
  the CPU approximation per the effect-split policy.
- The card-cover **`repeating-linear-gradient` hatch** → a flat dark fill.
- Avatar `oklch()` hues are converted to their nearest sRGB hex.

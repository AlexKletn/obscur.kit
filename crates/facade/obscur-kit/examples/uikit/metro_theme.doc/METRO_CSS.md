The complete **Metro / Modern UI** stylesheet (theme index 5) — the Windows 8
"Modern UI" look: white page, flat solid-color **tiles** with **sharp zero-radius
corners**, a bold blue accent (`#2d89ef`) plus orange/green/red/purple accents,
thin type, and a dark table header. The signature is the *absence* of rounding:
every surface is a hard rectangle. Same semantic classes and shared layout
numbers as [`MATERIAL_CSS`](super::material_theme::MATERIAL_CSS); solid fills only, faithful on CPU.

# Metro-defining choices against the shared view
- **`border-radius: 0` everywhere** except the radio (the one intentionally round
  control) and the circular spinner ring — matching the spec.
- Cards become flat `#f2f2f2` tiles with **no border and no shadow**.
- Toggles are **rectangular** (a bordered box + a sliding rectangular knob), not
  pills — Metro's on/off switch.
- Slider thumb is a tall thin **bar**, badges/avatars are hard color blocks, the
  table header is a solid **dark** band.

# Approximated
- Mono eyebrow **`letter-spacing`/`text-transform`** dropped (labels pre-cased).
- The card-cover live-tile hatch → a solid accent tile fill.

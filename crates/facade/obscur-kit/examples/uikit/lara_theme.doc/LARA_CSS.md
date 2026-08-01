The complete **Lara / PrimeNG** stylesheet (theme index 6) — the PrimeNG "Lara"
look: clean light page (`#f8f9fa`), white cards with a `1px #e9ecef` border, a
gentle shadow and a small 6px radius, a confident blue primary (`#3B82F6`), calm
greys (`#495057` text, `#6c757d`/`#9aa3ad` muted), soft `#ced4da`/`#dee2e6`
borders and filled inputs. Status badges are bold *solid* color blocks with white
text (Lara's tag style). Same semantic classes and shared layout numbers as
[`MATERIAL_CSS`](super::material_theme::MATERIAL_CSS); solid fills + soft shadows only, faithful on the CPU backend.

# Approximated (our engine's known gaps)
- Mono eyebrow **`letter-spacing` / `text-transform`** dropped (no engine
  support; labels are pre-uppercased in the view).
- Decisive **focus rings** (`0 0 0 3px #bfdbfe`) are interaction-state only and
  not shown on a still frame.
- The card-cover **`repeating-linear-gradient` hatch** -> a flat grey fill.

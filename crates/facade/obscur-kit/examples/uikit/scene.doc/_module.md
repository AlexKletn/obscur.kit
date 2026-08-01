The **imported UI-Kit** scene: a top *theme-switcher* bar over a *Material 3*
component gallery, authored as ONE theme-agnostic view (semantic classes) plus
a swappable per-theme stylesheet. Rendered headless to a PNG by the sibling
`uikit_png.rs` example.

# Architecture — one view, many stylesheets

[`view`] builds the switcher + gallery using only *semantic* class names
(`.card`, `.btn`, `.btn-filled`, `.chip`, `.badge`, `.avatar`, `.slider`, …) —
it never names a Material color, radius, or shadow inline. All appearance lives
in the stylesheets returned by [`themes`]. Swapping to a different design system
later is therefore *pure CSS*: add one more `(name, css)` entry to [`themes`]
and author its rules against the same class vocabulary. **All eight** entries
are now authored in full — from the **Material** entry Phase 1 shipped through
the **Fluent / Acrylic** entry (index 7), which layers translucent surfaces
over a "mica" page wash and frosts them with `backdrop-filter: blur()`.

# Fidelity notes (this is a faithfulness test of our CSS/layout/paint engine)

The imported design uses a handful of things our markup/engine cannot take
verbatim; each is approximated and documented here:

- **SVG icons** (FAB `+`, checkmarks, chevrons, kebab menu) → rendered as host
  font glyphs (`+`, `✓`, `▾`, `⋮`, `›`) — we never parse SVG.
- **Google Fonts** (Roboto / JetBrains Mono) → the host-provided CPU font
  (e.g. DejaVu). Text shapes and paints, just not in the exact face.
- **`letter-spacing`, `text-transform`, `line-height`** are not in our property
  set → dropped; eyebrow labels are authored already-uppercased.
- **`flex-wrap` / `flex: 1`** unsupported → rows use explicit widths + fixed
  flex tracks instead of wrapping.
- **Floating outlined-field labels** (label straddling the border) need overlap
  we don't do cleanly → labels are stacked above the value inside the box.
- **`repeating-linear-gradient`** cover → a plain `linear-gradient` approximation.
- **Two-layer `box-shadow`** IS supported (comma list) and used verbatim for the
  card elevation; GPU-class blur degrades to the CPU approximation per policy.
- The spinner is **static** (no animation on a still frame).

The **Browser Elements Wireframe Kit** scene: a grayscale, flat "spec sheet"
of every native HTML control shown across all of its visual states
(default / hover / focus / active / disabled / error / …). This is the
toolkit's **base-component / user-agent-stylesheet reference**, authored as one
theme-agnostic view built from *semantic* classes plus a single wireframe
stylesheet keyed to those classes (so [`stylesheet`] reads like a real UA sheet
whose state variants are ordinary CSS classes: `.wf-btn.hover`, `.wf-field.focus`,
`.wf-check.checked`, …). Rendered headless to a PNG by the sibling
`wireframe_png.rs` example.

# Fidelity notes — now authored on REAL primitives

An earlier pass of this sheet was a deliberate *gap-finding* exercise: six
base-element features the design leans on were only *approximated*. Those gaps
have since been closed, so this sheet now renders the AUTHENTIC CSS for each
(search "REAL" in the CSS below):

- **`outline` + `outline-offset` focus rings** — the focused control carries a
  real `outline: 2px dashed #2b2a27; outline-offset: 3px` on the `.wf-focus`
  wrapper; the ring paints OUTSIDE the box with a visible offset gap and takes
  no layout space (via the coordinator's outline path).
- **Dashed borders** (disabled controls, dashed dividers/underlines) — real
  `border: 1.5px dashed …`, stroked as actual dashes by both backends.
- **CSS-border triangles / arrows** (select `▼` / open `▲`, disclosure `▸` / `▾`,
  number-spinner `▲` / `▼`) — the classic `width:0; height:0; border-width:…;
  border-color:… transparent …` trick: three transparent sides + one inked side
  paint a true triangle (mitered trapezoid quads, transparent sides skipped).
- **Graph-paper page grid** — two crossing `repeating-linear-gradient()` layers
  in one `background-image` comma list over the page `background-color` (a real
  24px grid; GPU-faithful on Vello).
- **`repeating-linear-gradient`** hatches — the indeterminate progress hatch,
  the meter stripe fills, and the colour-swatch checker are real repeating
  gradients (GPU-faithful on Vello; flat first-stop on CPU, by policy).
- **Multi-line `<textarea>`** — a single real paragraph string that WRAPS inside
  the fixed-size box with a real `line-height` (no manual line splitting).
- **Inset `box-shadow` error ring** — a real `box-shadow: 0 0 0 1.5px #2b2a27
  inset` painted inside the field box.
- **`overflow: hidden`** — the open-select list, the table, the disclosure, the
  progress track and the colour swatch clip their children to their rounded
  corners.

Two things remain genuine gaps (NOT part of this batch, still approximated):
**`border-radius: 50%`** circles are given explicit half-px radii, and
**`letter-spacing` / `text-transform`** mono captions are authored already-
uppercased. The `✓` check, `✕` clear, `⌕` magnifier and `◣` resize grip stay as
host-font glyphs (they are not border-triangles).

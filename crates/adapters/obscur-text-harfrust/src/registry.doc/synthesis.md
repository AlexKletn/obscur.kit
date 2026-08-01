Decides whether a run resolved to `matched` (the nearest available face) must
be synthesized to satisfy the `requested` attributes (spec §5.18).

* **Oblique** — the request wants a non-upright slant (italic or oblique) but
  the matched face is registered upright: shear the glyphs to fake the slant.
* **Bold** — the request is bold-ish (weight ≥ [`BOLD_THRESHOLD`]) but the
  matched face is clearly lighter (weight < [`BOLD_THRESHOLD`]): thicken the
  glyphs to fake the weight.

When the matched face already carries the requested style neither flag is
set, so a true italic/bold cut renders as itself.

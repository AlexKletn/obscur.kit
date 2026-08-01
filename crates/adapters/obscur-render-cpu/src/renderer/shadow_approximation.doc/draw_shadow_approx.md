Rasterizes a CSS `box-shadow` as a **flat**, offset rounded rectangle at
reduced alpha — the software backend's blur-free approximation (spec §5.2).

The box's border box `rect` is translated by `offset` and inflated by `spread`,
then filled as a rounded rect (`radii`) in the shadow `color` at
[`SHADOW_APPROX_ALPHA`] of its alpha. The `blur` radius is **ignored** (this
backend has no gaussian blur), so the mark is a hard-edged, semi-transparent
rounded rect rather than a soft shadow — a documented approximation that keeps
device-free callers deterministic.

An **inset** shadow takes [`draw_inset_shadow_approx`] instead: it renders as an
inner *frame* (a rounded-rect ring) confined to the box, not an offset blob.
A degenerate (non-positive) rect draws nothing.

Sizes replaced elements whose width **and** height are both `auto` to
their content's intrinsic logical size, patching `layout_styles` in place
before the arrange pass (CSS Images 3 §5.4 default sizing algorithm; spec
§5.16).

A replaced element with neither dimension authored has no CSS length to
resolve — its box is a property of the decoded content, so it must be
measured. It reads the decoded representation's **logical** size (its pixel
size over its density, spec §5.16 DPI variants) from the already-resolved
image/vector caches — [`resolve_element_images`](Self::resolve_element_images)
and friends run once per frame *before* this, so the size is present — and
writes it as a concrete width/height so the box algorithms size the element
exactly as they would an authored `Npx`/`Npx`.

Only the both-`auto` case is handled here: an element with one axis
authored (the aspect-ratio-derived other axis) is a broader slice of §5.4
left for later. An intrinsic size with a zero or negative extent is
skipped, so a not-yet-decoded or degenerate asset leaves the element's
`auto` boxes untouched rather than collapsing them.

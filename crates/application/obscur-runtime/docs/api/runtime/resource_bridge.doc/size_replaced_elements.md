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

An auto size on a size-contained axis uses its contained intrinsic size, which defaults to zero.
Outside the contained axes, natural sizes are used when both axes are
auto. Deriving a size from the aspect ratio and a single authored axis
remains outside this pass. An intrinsic size with a zero or negative extent is
skipped, so a not-yet-decoded or degenerate asset leaves the element's
`auto` boxes untouched rather than collapsing them.

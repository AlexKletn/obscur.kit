End-to-end verification that **authored CSS grid** reaches the layout engine
and produces the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox)
geometry through the *real* live pipeline (spec §4.1/§4.2).

Every test drives the genuine path — author CSS text (`display: grid` with
`grid-template-columns`/`-rows`, `gap`, `repeat(…)`, `minmax(…)`), compile it
through the real `obscur-css` frontend (`source -> parse -> compile ->
matchable Stylesheet`), and render one frame on a [`UiRuntime`] via
[`render_now`](UiRuntime::render_now), which runs `reconcile -> match ->
cascade -> resolve (ComputedStyles) -> grid-templates -> layout
(LayoutBoxes)`. The assertions read the *actual* absolute [`LayoutBox`]
coordinates the pipeline produced (`runtime.layout_boxes()`), never a
hand-built `GridTemplate`/`LayoutStyle`, so a regression anywhere in the chain
(CSS lowering → resolver → `grid_template_for` conversion →
`GridLayout::with_template` dispatch → the track-sizing algorithm) fails the
run.

# Container sizing

A grid container honours its own explicit `width`/`height`: [`GridLayout`]
sizes its tracks (and distributes `fr` space) against the authored extent, and
the container box *is* that size. A container with no explicit size fills the
available extent its parent flow hands down (the viewport) — the
fill-available default. Tests that author an explicit container size render
under a larger viewport and assert the container is its authored size;
`auto_placement_wraps_to_second_row` keeps a viewport-driven (auto) case to
prove the fill-available default still holds.

# Proven vs still limited

Proven end-to-end here: fixed / percentage / `fr` / `auto` / `minmax` column
tracks, `fr` free-space distribution, `gap` on both axes, `repeat(N, …)`
expansion, `repeat(auto-fill, …)` sized from the container extent, row-major
auto-placement into implicit rows, and explicit fixed row tracks. Percentage
tracks resolve against the container's own content extent inside the algorithm
(`30%` of a `400px` container → `120px`). Still out of scope (documented in
steps 1-2, not exercised here): line-based placement / spanning, dense packing,
named lines, and alignment of an item within its track.

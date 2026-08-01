End-to-end verification that **authored CSS grid line placement** (`grid-column`
/ `grid-row`, integer lines and `span`) reaches the layout engine and produces
the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox) geometry through
the *real* live pipeline (spec §4.13, the positive-line + span subset).

Each test drives the genuine path — author CSS text (`display: grid` plus
`grid-column` / `grid-row` shorthands on the children), compile it through the
real `obscur-css` frontend (`source -> parse -> compile -> matchable
Stylesheet`), and render one frame on a [`UiRuntime`] via
[`render_now`](UiRuntime::render_now), which runs `reconcile -> match -> cascade
-> resolve (ComputedStyles, incl. the four `grid-*-start`/`-end` longhands) ->
grid-templates -> layout (LayoutBoxes)`. The assertions read the *actual*
absolute [`LayoutBox`] coordinates the pipeline produced
(`runtime.layout_boxes()`), never a hand-built `GridPlacement`/`LayoutStyle`, so
a regression anywhere in the chain (CSS shorthand→longhand lowering → resolver
`GridLineValue` → coordinator `grid_placement_of` → the grid algorithm's
two-phase placement) fails the run.

Every container authors an explicit `width` so the columns (and thus the placed
item x-coordinates) are deterministic regardless of the viewport.

# Proven vs still limited

Proven end-to-end here: line-based column placement (`grid-column: 1 / 3`),
`span` on an auto-placed item (`grid-column: span 2`), a single explicit line
(`grid-column: 2`) with auto items flowing around it, explicit row placement
(`grid-row: 2`) against fixed row tracks, a span crossing an interior `gap`, a
mixed explicit + auto layout where the auto items first-fit around the occupied
cell, negative (from-end) lines (`grid-column: 1 / -1` spanning the whole row),
the `grid-area` four-line shorthand, and **named-area placement**
(`grid-template-areas` on the container + `grid-area: <name>` on the items,
resolved in the runtime against the parent's parsed template), and **named grid
lines** (`[name]` groups in the container's `grid-template-columns` +
`grid-column: <name> / <name>` on the item, resolved in the runtime against the
parent's per-axis line-name map).

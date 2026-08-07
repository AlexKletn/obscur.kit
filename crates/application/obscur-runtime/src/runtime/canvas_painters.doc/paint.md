Runs each painter against the laid-out box of every element that names one, and
returns what each drew.

"Every element that names one" is the whole walk: since `-obscur-painter` a
painter is not a `<canvas>` facility, so the loop is over live nodes and the
question asked of each is `painter_key` — the node's own `canvas_painter` prop
first, then the computed `-obscur-painter` name. An application that registered
no painter pays one branch and returns.

A node with no box is skipped rather than painted at the origin — a
`display:none` branch has nowhere to paint into, and guessing would put its
drawing in the corner of the window.

Runs every canvas's painter against its laid-out box, and returns what each drew.

A node with no box is skipped rather than painted at the origin — a
`display:none` branch has nowhere to paint into, and guessing would put its
drawing in the corner of the window.

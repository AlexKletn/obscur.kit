Sets the viewport scroll offset `(x, y)` in logical pixels — the
document-coordinate point at the top-left of the viewport (spec §4.1).

The single scroll container is the viewport. There is no live scroll-gesture
system yet, so this is the test/host-driven input that engages
`position: sticky` clamping on the next frame: at `(0.0, 0.0)` (the default)
every sticky node lays out at its static flow position; a non-zero offset
scrolls the document under the viewport, so a sticky node sticks to the
scrollport edge once scrolled past its inset threshold. Wiring a real scroll
gesture to this setter is a separate, later step.

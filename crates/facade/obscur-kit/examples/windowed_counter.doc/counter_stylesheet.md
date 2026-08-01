The authored stylesheet: one rule per class, each painting a background +
size. Layout is flow (vertical stacking) — `display` is not yet mapped and
the property set has no margin/align — so the bars stack top-to-bottom
inside the dark `.counter` panel.

Text color defaults to white in the scene coordinator, and `color` does not
reliably inherit to the child text node in the current cascade, so rather
than author a per-label text color every bar is given a dark-enough
background for white glyphs to read: `.label` is a dark slate, and the red
`.decrement` / green `.increment` buttons already carry white text well.

The solid brush the display list paints `text` with, as an `(r, g, b)` triple.

Reads the paint output rather than a computed style, because the placeholder
has no node to compute a style on: it is a generated run in the display list
and nothing else.

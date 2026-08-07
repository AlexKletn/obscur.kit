A number where the standing command takes none.

`Z` closes a subpath and takes no arguments, and SVG's grammar puts no implicit
command after it — so the `1` in `M0 0 Z 1` belongs to nothing. It is refused
for the same reason a number before any command is: guessing a verb for it would
turn a typo into a drawing.

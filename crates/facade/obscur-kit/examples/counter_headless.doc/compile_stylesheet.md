Compiles [`STYLESHEET`] through the CSS frontend (`source -> parse ->
compile`) and prints a one-line summary of the resulting native structures.

This proves the styling frontend composes end-to-end and that recognised-but-
unsupported features surface as diagnostics rather than errors or panics.

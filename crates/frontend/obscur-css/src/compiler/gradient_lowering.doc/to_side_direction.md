Resolves a `to <side>` / `to <corner>` keyword list (the tokens after `to`)
into a [`GradientDirection`]. A single side maps to its axis angle; a two-keyword
corner maps to a [`Corner`] (carrying its nominal diagonal angle, resolved
aspect-ratio-aware at paint time). Returns [`None`] for an empty or unrecognised
list.

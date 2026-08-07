A slider's position and the scale it sits on, read from the same
`slider-low`/`slider-high`/`slider-min`/`slider-max` properties the drag
path and the painter read — so what is announced is what is drawn.

A control that states no `low` states no position, and answers
[`SemanticValue::None`] rather than inventing the minimum: a bare
`<slider/>` has not been given a value, and reporting `0` would be a claim
the tree never made.

A RANGED slider answers [`SemanticValue::Range`], because it has two
positions and neither of them is "the value".

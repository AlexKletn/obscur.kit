Takes the reactive set out, leaving none behind.

The reactive frame path needs the sheets as a slice *while* it holds
`&mut self` for the render, so it moves them out and puts them back with
[`set_reactive`](Self::set_reactive) — restoring them is installing them,
which is why there is no second setter for it.

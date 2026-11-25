The block for one `style` string, compiling it the first time it is seen.

`None` when no compiler is installed. Compiled once per distinct STRING rather
than once per node: a virtualized list of a thousand rows carrying the same
`style` pays for one parse.

The faults go to the sink on that same first compile, so a misspelled property
is reported once per distinct attribute — not once per node, and not once per
node per frame.

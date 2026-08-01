**The case that was impossible.** One symbol, two nodes, one handler — and the
handler is told which node the user hit.

This is the generalisation of `on_row_click`'s own argument: a per-row
`on:click` "cannot carry this, because every row a builder emits shares one
command symbol". That reasoning applies to every event, and one bespoke
signature per case was the only answer available.

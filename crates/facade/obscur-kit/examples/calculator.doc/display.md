The current entry, with the pending left-hand side above it.

Each line is a flex row that justifies its text to the end. `text-align: right`
would do the same thing now that alignment resolves against the containing
block rather than the shrink-wrapped text box; flex justification is kept
because it also decides the vertical centring these rows need.

Fill this answer's gaps from a weaker one, field by field.

What a composite source does: ask the portal, then let `gsettings` supply what
the portal does not carry, then `kdeglobals` the palette neither of them has.
`self` wins wherever it has an answer.

Field by field rather than whole-struct, and the palette too: two sources each
knowing half a palette is the ordinary case, and taking the first non-empty
palette entire would throw the other half away.

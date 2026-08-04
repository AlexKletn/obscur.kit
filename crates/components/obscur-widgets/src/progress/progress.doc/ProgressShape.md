The shapes this widget ships.

Both are PATHS, not two drawing routines: the painter strokes whichever it is
given and trims the same path for the fill, so the two differ only in geometry.
That is what makes a third shape a matter of supplying a path rather than of
writing another branch — see the module ledger for what supplying one still
needs.

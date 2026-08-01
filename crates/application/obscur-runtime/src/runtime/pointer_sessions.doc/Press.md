One outstanding press: a `Down` with no matching `Up` or `Cancel` yet, held
per [`PointerId`] so concurrent pointers do not overwrite each other.

A mouse can only ever have one of these; a touchscreen has one per finger,
and they are genuinely independent — one finger pressing a button must not
retarget, activate or cancel what another finger is holding.

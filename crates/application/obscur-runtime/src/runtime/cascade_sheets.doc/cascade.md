The ordered sheet list the matcher runs on: the UA default first, then
`author`.

First means *matched* first, not strongest: the UA sheet carries
[`CascadeOrigin::SystemDefault`](obscur_style::cascade::CascadeOrigin::SystemDefault),
the lowest origin, so any author declaration of the same or higher
specificity overrides it (§4.14) — the order only settles ties inside one
origin. With no UA sheet installed the author list is returned unchanged,
which is what keeps matching byte-identical for a runtime that never
installs one.

The hub's lifecycle record for the resource `url` names — `loading`,
`ready` (with its metadata) or `failed` — or [`None`] when nothing has ever
requested it (spec §5.15).

The two halves of that answer live in two maps that must agree: the handle
minted for the URL, and the state the hub holds for that handle. Asking
through one method is what keeps a caller from reaching past the first to
the second and reading a handle the cache has since released.

Toggles `node`'s membership: adds it if absent, removes it if present.

Returns the resulting membership — `true` if `node` is now selected,
`false` if it was just removed. Adding goes through [`add`](Self::add)
(so the lead follows); removing goes through [`remove`](Self::remove).

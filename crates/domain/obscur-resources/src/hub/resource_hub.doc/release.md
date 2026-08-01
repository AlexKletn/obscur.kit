Releases one reference to `handle`, evicting the resource once
unreferenced (spec §5.15).

Decrements the reference count; when it reaches `0` the entry and its
deduplication index are removed and `true` (removed) is returned.
Otherwise the resource is still shared and `false` is returned. An unknown
handle returns `false`.

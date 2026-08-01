Returns the cached style for `node` only if it was computed at exactly
`version`; a stale (older or newer) stored version misses (§4.3).

Always `None` when the cache is disabled.

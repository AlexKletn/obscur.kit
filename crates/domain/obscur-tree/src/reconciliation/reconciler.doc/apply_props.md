Refreshes a node's CSS-relevant style attributes from a desired element's
props, returning `true` if they actually changed. A no-op (returning
`false`) if `id` is stale or the extracted attributes equal the current ones.

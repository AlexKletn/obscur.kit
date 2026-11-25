Reconstructs the location as `path?query#fragment`.

The `?` and query are emitted only when the query is non-empty, and the
`#` and fragment only when a fragment is present. Query pairs are joined
with `&` and each rendered as `key=value`.

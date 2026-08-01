Recursively inserts `id` and its descendants into `semantic`, linking each
under `parent` and folding `id`'s own structural state into the
`inherited` state its subtree sees. The `visited` set guards against a
malformed (cyclic) retained tree by inserting each node at most once.

Authenticates `token`, returning the scopes granted to it.

Returns [`None`] for an unknown token (including any token when the
registry is empty). For a known token the registered scopes are returned
with any production exclusions (`Control`/`Inject`) removed.

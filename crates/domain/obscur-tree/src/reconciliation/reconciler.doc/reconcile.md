Reconciles the children of an existing retained `parent` against a desired
fragment, updating the tree and returning the op log.

If `parent` is not a live node the pass is a no-op and an empty plan is
returned.

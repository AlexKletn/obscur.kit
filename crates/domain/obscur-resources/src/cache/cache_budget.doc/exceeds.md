Returns whether the cache is already over budget and needs eviction (spec
§5.19 eviction).

This is `true` when the current bytes or entries strictly exceed their
respective caps. A disabled cache is not "over budget" in this sense — it
simply never admits — so this reports only the size limits.

The §4.12 parse cache, keyed by `(SourceId, SourceVersion)`. Retained
across installs: on a reload of N sources where one changed, only that
one is re-parsed.

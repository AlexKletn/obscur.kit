Returns a handle to this runtime's application fallback registry.

The returned handle *shares* the registry, so registering through it
reaches this runtime — and every other runtime given the same handle.

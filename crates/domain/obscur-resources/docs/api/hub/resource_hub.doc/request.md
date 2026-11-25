Requests a resource, returning its [`ResourceHandle`] (spec §5.15
deduplication).

If an entry already tracks the request's `(source, kind)` identity, this
[`retain`](ResourceEntry::retain)s it and returns the existing handle so
callers share one resource. Otherwise it allocates a fresh identity,
inserts a new [`ResourceEntry`] (pending, initial version, reference count
`1`), indexes it for future deduplication, and returns the new handle.

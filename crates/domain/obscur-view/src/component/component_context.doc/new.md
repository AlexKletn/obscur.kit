Creates a context over the given subscription `scope`, lifecycle
`phase`, and `needs_update` flag.

The runtime constructs this immediately before calling `render` or a
lifecycle hook and drops it immediately after, so the borrows are held
only for the duration of that call.

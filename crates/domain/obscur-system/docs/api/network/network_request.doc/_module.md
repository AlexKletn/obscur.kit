An HTTP-like network request a host loader fulfils (spec §5.15).

The toolkit performs no network I/O of its own: a [`NetworkRequest`] is a
pure, backend-agnostic *description* of a fetch that the host resolves
through its own loader (spec §5.15). It carries an HTTP-shaped method, a URL,
ordered headers, and an optional body — never a live connection.

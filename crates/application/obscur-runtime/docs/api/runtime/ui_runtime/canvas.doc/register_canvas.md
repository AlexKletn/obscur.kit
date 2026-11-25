Registers the painter a `<canvas>` naming `id` will be drawn by.

The same shape as `register_virtual_list`, and for the same reason: a `ViewNode`
is data and cannot carry a closure, so the node names a symbol and the runtime
holds what the symbol means. Registering over an existing name replaces it.

The painter runs once per composed frame, after layout — so it knows its own
box — and before compose, into a region clipped to that box and translated to
its origin.

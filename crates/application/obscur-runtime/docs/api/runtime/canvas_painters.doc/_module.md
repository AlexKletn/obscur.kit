The painters an application registered, and the one place they run.

Drawing is a CALLBACK, not a resource, and that is what decides the shape here.
An image or a vector reaches paint through a map keyed by `src` because it is an
asset the runtime resolved; a canvas painter is application code the runtime
CALLS, so it needs `&mut` and it needs a registration point. The precedent is
`register_virtual_list(id, builder)` — a `ViewNode` is data and cannot carry a
closure, so the node names a symbol and the runtime holds what the symbol means.

It runs before compose rather than inside it. The compose walk takes `&self` and
is handed maps it only reads; calling a painter from inside it would make the
whole walk `&mut` for the sake of the rarest node in the tree. So painters run
once against the laid-out boxes and leave behind a `NodeId -> commands` map that
compose splices like any other resolved input.

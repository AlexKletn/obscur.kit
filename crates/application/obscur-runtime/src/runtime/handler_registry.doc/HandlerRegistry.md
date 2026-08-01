Every binding the runtime dispatches through: the command handlers at each
§2.6 link, the §2.3 listeners, the open focus scopes, the editable-field
handlers, and the shortcut/descriptor tables §6.7 gates a dispatch on.

# Why these ten tables are one entity

They are not ten independent registries; they are one answer to "who runs
when this command arrives", split by *where* the binding sits. Routing reads
four of them in a fixed order in a single walk
([`dispatch_along`](Self::dispatch_along)), and two pairs must be maintained
together or a stale binding survives its owner:

- closing a focus scope has to drop that scope's handlers, or a closed
  dialog's commands keep routing ([`close_scope`](Self::close_scope));
- clearing a node has to drop both its handlers **and** its listeners, which
  is why [`clear_node`](Self::clear_node) is one operation rather than two a
  caller could half-perform.

# What it deliberately does not do

It never touches the tree, and it never asks for a frame. Building the route
needs the node path, and deciding that a handler having run means the next
frame must rebuild is the runtime's call, not a table's — so the runtime
hands the route in and reads the "did anything run" answer back.

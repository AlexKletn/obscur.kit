A LIFO stack of saved focus for scope enter/exit (spec §7.1).

When a focus scope (a dialog or menu) opens, the runtime saves the node that
was focused just before it. When the scope closes, that node is restored.
Because scopes nest, saves and restores are last-in/first-out: closing scopes
restore focus in the reverse order they were opened.

A frame holds `Option<NodeId>` rather than `NodeId`, and that is the whole
difference between a stack that can be used and one that cannot: a scope opening
over an unfocused surface must push a frame that says "nothing", so that the
pairing of pushes to pops stays one-to-one however the scopes nest.

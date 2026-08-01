A LIFO stack of saved focus for scope enter/exit (spec §7.1).

When a focus scope (a dialog or menu) opens, the runtime saves the node that
was focused just before it. When the scope closes, that node is restored.
Because scopes nest, saves and restores are last-in/first-out: closing scopes
restore focus in the reverse order they were opened.

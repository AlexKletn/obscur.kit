Maps shortcuts to command symbols and resolves key input (spec §6.7).

A `CommandRouter` holds an ordered list of shortcut→command-symbol bindings.
Registration order is priority: [`resolve`](CommandRouter::resolve) returns
the command symbol of the *first* binding whose shortcut matches the key
input, so earlier bindings shadow later ones. The router stores only the
app-owned command [`InternedSymbol`]s — the *same* symbols the application
registers handlers for and binds `on-click`/`on-change` events to — so
keyboard input and pointer input dispatch through one command identity. It
does not own the [`Command`](crate::command::Command)s themselves.

Returns whether two handles share the same underlying registry.

True exactly when one was [`Clone`]d from the other — the property that
makes an `AppCommands` an *application* level spanning several windows,
rather than a second per-window table.

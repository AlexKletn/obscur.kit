Derives an item from a unified [`Command`] and its live [`CommandState`]
(spec §6.7).

The item is a *projection* of the one action descriptor, not a second
copy of it: the label is the command's [`title`](Command::title), the
fired command is its [`id`](Command::id) — the same symbol its shortcut
resolves to and its handler is registered for — and the item is enabled
exactly while the state is. Editing the command edits the menu.

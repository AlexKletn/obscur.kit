Pushes a new entry for `route` with `state`, assigning a fresh id.

Standard browser semantics: pushing truncates any forward history (every
entry after the current cursor) before appending, so a push after going
back discards the entries that were ahead. The new entry becomes current.

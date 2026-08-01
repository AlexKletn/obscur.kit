Replaces the current entry in place with a new entry for `route` and
`state`, assigning a fresh id.

The cursor and the rest of the stack are unchanged; only the current
entry is overwritten. When the history is empty there is no current entry
to replace, so this behaves like [`push`](HistoryProvider::push).

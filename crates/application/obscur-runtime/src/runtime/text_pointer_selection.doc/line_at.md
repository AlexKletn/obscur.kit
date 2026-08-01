The whole logical line containing byte `offset` in the focused editable
(triple-click, spec §5.11), bracketed with the buffer's
`NavigationUnit::LineBoundary` engine — the byte span between the enclosing
`'\n'` boundaries (or the whole text in a single-line field). Returned as a
forward `[start, end)` for the caller to set through its non-mutating edit
path.

Applies a range gesture over `range` to `selection`.

- When the mode [`allows_multiple`](SelectionMode::allows_multiple),
  replaces the selection with `range`.
- For [`Single`](SelectionMode::Single), a range collapses to its last
  item (selected on its own).
- For [`None`](SelectionMode::None), selection is disabled, so this is a
  no-op.

Applies a click on `node` to `selection`.

`toggle_modifier` is `true` when the click carried a toggle modifier
(Ctrl/Cmd). The behaviour depends on the mode:

- [`None`](SelectionMode::None): selection is disabled, so this is a
  no-op.
- [`Single`](SelectionMode::Single): replaces the selection with `node`.
- [`Multiple`](SelectionMode::Multiple): if `toggle_modifier` or
  [`toggle_on_click`](Self::toggles_on_click) is set, toggles `node`;
  otherwise replaces the selection with `node`. When `allow_empty` is
  `false`, a toggle that would remove the only selected item is
  suppressed so at least one item stays selected.

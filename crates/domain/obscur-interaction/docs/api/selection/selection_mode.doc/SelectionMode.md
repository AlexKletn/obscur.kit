How many items a [`Selection`](super::Selection) may hold at once.

This is the *item* selection model — selecting list rows, canvas objects, or
tree nodes — and is distinct from text-caret selection in `obscur-text`.

- [`None`](Self::None) disables selection entirely.
- [`Single`](Self::Single) allows at most one selected item.
- [`Multiple`](Self::Multiple) allows many selected items and supports
  range (shift) extension.

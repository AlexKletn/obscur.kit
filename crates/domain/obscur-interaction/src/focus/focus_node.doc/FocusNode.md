A node's focus properties within the focus model (spec §7.1).

Each participating node contributes a `FocusNode` describing whether it can
receive focus, its position in the tab sequence, and whether it is currently
disabled. The `tab_index` mirrors HTML `tabindex`: a negative value marks the
node focusable programmatically but excluded from tab navigation, `0` places
it in natural (registration) order, and a positive value gives it an explicit
priority that sorts ahead of the natural-order nodes.

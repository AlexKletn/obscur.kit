The set of currently selected items plus the range-selection anchor.

The selected items live in a [`BTreeSet`], so iteration is deterministic
(ascending [`NodeId`] order) regardless of the order they were selected in.
The optional [`SelectionAnchor`] records the fixed anchor and moving lead
used to extend a range with shift-selection; it is `None` exactly when the
selection is empty.

This is the *item* selection model — list rows, canvas objects, tree nodes —
and is distinct from text-caret selection in `obscur-text`. Higher-level
rules (mode, toggle-on-click, non-empty enforcement) live in
[`SelectionPolicy`](super::SelectionPolicy); this type is the raw mutable
state it drives.

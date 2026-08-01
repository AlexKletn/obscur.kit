Replaces the selection with exactly `range`, setting the anchor to span
from the first to the last element.

An empty `range` clears the selection and the anchor. This models a
shift-selection: the anchor's `anchor` is the first item and its `lead`
is the last.

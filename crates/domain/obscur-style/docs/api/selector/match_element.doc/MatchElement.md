An element the selector matcher can navigate and inspect.

This is the seam that keeps `obscur-style` a pure domain crate: the matcher
operates only through this trait, so a retained-tree node (or any other
element graph) can implement it in a later sub-step without this crate
depending on the tree. Names and attributes are exchanged as `&str` — not
interned symbols — because CSS and the tree intern into different tables;
see the crate's interner-coherence note.

Navigation methods ([`parent`](Self::parent),
[`previous_sibling_element`](Self::previous_sibling_element)) return owned
`Self` values: implementors are expected to be cheap `Copy`/handle/cursor
types over a shared arena rather than owning the node data.

Only element siblings are considered "siblings" here; text and other
non-element children are excluded from indices and counts.

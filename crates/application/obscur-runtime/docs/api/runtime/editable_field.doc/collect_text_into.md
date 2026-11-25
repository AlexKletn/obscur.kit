Appends the text content of `node` and its descendants into `out`
(depth-first, in child order), bounded by the tree so a malformed subtree
terminates.

The `.placeholder` overlay is **skipped** in its entirety: it is a
non-editable rendering layer (the CSS `::placeholder` pseudo-element, shown
while `:placeholder-shown` holds), *not* the field's value, so it must never
be seeded into the editing buffer. An empty placeholdered field therefore
collects no text — it focuses to an empty buffer, caret at 0.

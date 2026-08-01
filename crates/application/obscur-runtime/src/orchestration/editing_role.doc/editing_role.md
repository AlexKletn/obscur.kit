Classifies `node` relative to the editing field `field`: the field's own text,
its `.placeholder` overlay, or neither — found by walking `node`'s
logical-parent chain toward the root.

The editing session is bound to the editable *field* (the ancestor that binds
`EVENT_INPUT`), but the glyphs live on a `Text` descendant; this maps that
descendant back to its field so the compose/measure phases can override its
content and paint the caret/selection there. The walk is bounded by the tree
size, so a malformed (cyclic) parent chain terminates deterministically instead
of looping.

A node reached through a `.placeholder` overlay (CSS `::placeholder`) on the way
up to `field` is classified [`EditingRole::Placeholder`] rather than `Value`:
the overlay is the `:placeholder-shown` hint, not the field's editable text, so
the session overrides it only while it has typed text (see
[`EditingRole::overrides`]) and, when it does, paints in the field's ink — not
the overlay's muted color.

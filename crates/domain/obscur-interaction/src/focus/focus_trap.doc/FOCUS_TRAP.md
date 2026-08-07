Marks the element whose subtree traps focus.

While an element carrying this property is in the committed tree, the runtime
holds an open trapping [`FocusScope`](super::FocusScope) over that element and its
descendants: Tab and Shift+Tab cycle inside it, a pointer press outside it moves
no focus, and a programmatic focus request for a node outside it is refused. When
the element leaves the tree the scope closes and focus returns to whatever held it
before the trap opened.

The value is read for truth, not for presence: `Bool(false)` declares a dialog
that is *not* modal and traps nothing, which is what lets a widget emit the
property unconditionally and carry the author's answer in it. Every other value —
including `Null`, the bare-attribute spelling — traps.

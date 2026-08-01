The [`ComponentId`] of the built-in this tag names, or [`None`] when it names
no built-in.

Built-in widgets and non-container primitives resolve to their stable
`COMPONENT_ID`; the lowercase `button`/`img`/`canvas`/… element spellings
resolve to the same widget or primitive as their PascalCase component
spelling.

[`OUTLET_TAG`] is the one entry here that is not a component at all: nothing is
ever registered under [`SLOT_OUTLET`], and no node ever retains it — projection
substitutes it away inside a component's render, and the reconciler refuses it
anywhere else. It is a built-in because it is a *reserved tag*: `<slot>` must
mean the outlet rather than a `div` named `slot`, and this function is where
every reserved tag is stated.

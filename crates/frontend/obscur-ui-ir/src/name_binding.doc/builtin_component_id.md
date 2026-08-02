The [`ComponentId`] of the built-in this tag names, or [`None`] when it names
no built-in.

Built-in widgets and non-container primitives resolve to their stable
`COMPONENT_ID`; the lowercase `button`/`input`/`textarea`/`checkbox`/`radio`/
`select`/`table`/`img`/`canvas`/… element spellings resolve to the same widget
or primitive as their PascalCase component spelling, so markup and the Rust
builder API produce **one node** rather than two that merely look alike.

`textarea` is the one entry with no component spelling of its own: HTML's two
text-entry tags are one widget here, and which of them a node is stays readable
from its local name (`EditableField::is_multiline` reads
`local_name() == "textarea"`), not from a second id.

[`OUTLET_TAG`] is the one entry here that is not a component at all: nothing is
ever registered under [`SLOT_OUTLET`], and no node ever retains it — projection
substitutes it away inside a component's render, and the reconciler refuses it
anywhere else. It is a built-in because it is a *reserved tag*: `<slot>` must
mean the outlet rather than a `div` named `slot`, and this function is where
every reserved tag is stated.

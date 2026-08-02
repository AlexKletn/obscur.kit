The slot this node is an outlet for: the value of its `slot="…"`, or the
empty name — the **default slot** — when it is an outlet naming none.

The empty name is also the answer for a node that is not an outlet at all,
which no caller asks about: every call site tests [`is_outlet`] first.

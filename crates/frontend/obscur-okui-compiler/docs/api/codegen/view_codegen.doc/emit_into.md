Appends the statements that push `nodes` into the enclosing `__nodes` sink,
at `indent` levels (4 spaces each).

# Conditionals are grouped, not emitted one by one

`<If>` / `<ElseIf>` / `<Else>` arrive as *siblings*, but they are one
construct: emitting each on its own would produce `if a {…} if b {…} {…}` —
three independent statements, the last of which always runs. So a run of them
is collected and emitted as a single `if / else if / else` chain, which is the
only shape where exactly one branch renders.

A run ends at the first non-conditional sibling, or after an `<Else>` (nothing
may follow the fallback). An `<ElseIf>` / `<Else>` with no `<If>` before it
starts no chain and is emitted as its own branch, which is what a stray
`<Else>` deserves: it renders unconditionally rather than silently attaching
to an unrelated `<If>` further up.

# The inherited key

`key` is the identity a `<For>` gives the node it renders for one item. It is
threaded down rather than applied at the loop, because the item's root may sit
under a conditional: `<For><If cond={x}><Row/></If></For>` must still key the
`Row`. A node with a key of its own keeps it; `None` outside a loop leaves
every node exactly as before.

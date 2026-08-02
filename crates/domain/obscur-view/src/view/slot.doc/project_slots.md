Projects `content` — a component host's authored children — into the outlets
declared by `output`, the component's own render (spec §2.12).

Each outlet is replaced, in place and in order, by every node of `content`
assigned to the slot it names: by a `slot="…"` that names it, or — for the
outlet that names no slot, the **default** one — by carrying no assignment at
all. A child assigned to a slot the output declares no outlet for reaches no
position and does not appear; the compile-time rule is what refuses a
`slot="…"` naming an undeclared slot, and it has already run.

An outlet nothing is assigned to is replaced by its own children: its
**fallback**, the content the component renders when the author supplied
none. An outlet with no fallback and nothing assigned is replaced by nothing
at all, rather than by a leftover box.

`output` is returned untouched, without walking or rebuilding it, when it
declares no outlet — which is every component that takes no children.

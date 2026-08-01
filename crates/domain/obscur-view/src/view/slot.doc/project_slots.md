Projects `content` — a component host's authored children — into the outlets
declared by `output`, the component's own render (spec §2.12).

Each outlet is replaced, in place and in order, by every node of `content`
whose `slot="…"` names it. An outlet nothing names is replaced by nothing.
A child assigned to a slot the output declares no outlet for reaches no
position and does not appear — the compile-time rule is what refuses a
`slot="…"` naming an undeclared slot, and it has already run.

`output` is returned untouched, without walking or rebuilding it, when it
declares no outlet — which is every component that takes no children.

The panel bound to a click command, with only `b` binding one of its own.

The arrangement a bubbling test needs: the container binds `on-click`, `a` binds
nothing at all, and `b` binds its own command. A click on `a` therefore has no
binding to resolve at the node it hit and must walk up to the panel — which is
what makes `target` and `current_target` two different nodes, the property the
field exists for. A click on `b` resolves at the node it hit, so the same fixture
also supplies the control case where the two coincide.

The panel's own binding is also what an activation reaches when focus sits on
`b`: [`command_view`](Self::command_view) would answer the same question from a
child, but only this arrangement can say whether the answer came from the node or
from its ancestor.

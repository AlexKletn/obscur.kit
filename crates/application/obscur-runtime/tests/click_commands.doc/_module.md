A click resolves a command, and the event says which node it was attached to
(spec §2.3, §2.6).

Eight tests about the step between "a pointer went down and came up on the same
node" and "an application handler ran". A click is not a pointer event: it is the
*pair*, and the binding it resolves may live on the node that was hit or on any
ancestor of it, so the same gesture answers two different questions —
`target`, the node the pointer hit, and `current_target`, the node whose binding
claimed the command.

The fixture exists to keep those two apart. `InteractionPanel::ancestor_bound_view`
binds the container and `b` but leaves `a` unbound, so a click on `a` bubbles and
the two fields provably differ, while a click on `b` resolves where it landed and
they provably coincide. A test that only checked the field exists would pass with
`current_target` hard-wired to `target`; these do not.

The rest are what must *not* dispatch: a press and a release on different nodes
is a drag, not a click; a `Move` over a bound subtree resolves nothing at all; a
binding with no registered handler still resolves its symbol but runs nothing and
panics on nothing. The last test is the frame consequence — a dispatch on its own
requests a re-render, because the handler it just ran is presumed to have changed
the state the view is built from.

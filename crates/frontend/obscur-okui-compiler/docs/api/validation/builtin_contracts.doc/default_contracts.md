Every contract a compile checks against when the caller registers none —
the primitives, the widgets, and the base-element child policies, in that
order.

It lives here rather than in `ContractRegistry::builtins` because the answer
to "what does a default compile know about?" is a property of this table, not
of the registry that happens to load it. Later entries win on a name clash,
which is why `mounted_element_contracts` is last: its lowercase tags collide
with nothing above, and if one ever did, the tag whose factory owns its
children is the one that must be believed.

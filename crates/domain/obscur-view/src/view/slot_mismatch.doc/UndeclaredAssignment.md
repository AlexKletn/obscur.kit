An authored child names a slot the component's contract does not declare, so
projection drops it.

Carries the empty name when the content named no slot at all and the component
declares no default slot — the loose-child case, which is the same fault without
a spelling to point at.

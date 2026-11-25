Adds `tag` to the allowed list, switching to [`Only`](Self::Only) from
either other policy.

Naming an allowed child is how a contract *turns on* child restriction, so
this necessarily leaves [`Any`](Self::Any); it also leaves
[`None`](Self::None), because "no children, except this one" is a
contradiction and the later, more specific statement is the author's
intent.

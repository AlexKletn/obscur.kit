How many positional registrations are still waiting for their name to resolve.

A name that matches nothing costs one tree walk per committed frame and never
fires; a non-zero count that never falls is the symptom, and
[`bound_node`](Self::bound_node) says which name it is.

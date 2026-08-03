Every reserved event name, in one place.

It exists so the collision checks are written over the set rather than over a
list of pairs that has to be extended by hand each time a name is added — the
shape the pairwise version had already grown into, where forgetting one line
silently skipped a check.

A name outside this list is an *author-hashed* symbol: `on:whatever="…"` binds
fine and nothing ever dispatches it. That is the trap this list makes
enumerable — a front end can check an `on:` name against it and say so.

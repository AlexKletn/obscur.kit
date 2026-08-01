`font-weight: bolder` / `lighter` against the parent's computed weight
(§4.14): [`resolve_relative_weight`].

A relative weight never becomes a number at parse time: CSS defines it as a
fixed nine-bucket table keyed on the **parent's** computed weight, so it can
only be resolved at the computed-value step, with the parent in hand. That
table, the bucket rounding it needs and the parent reading it is keyed on are
this file; the resolver asks it one question.

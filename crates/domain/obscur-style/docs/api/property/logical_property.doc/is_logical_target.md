Whether any logical property can resolve *onto* this one.

The inverse question to [`physical_under`](Self::physical_under), and the point
of asking it is that the answer is no for about eighty of the hundred computed
properties. `cascaded_for` used to scan all twenty logical properties for every
one of them, which is two thousand mappings per node per resolve to discover
that `color` has no logical spelling.

`the_two_directions_of_the_logical_mapping_agree` keeps this in step with
`physical_under` by construction: it is a test over every logical property in
every writing mode, so a new mapping cannot be added without this answering for
its target.

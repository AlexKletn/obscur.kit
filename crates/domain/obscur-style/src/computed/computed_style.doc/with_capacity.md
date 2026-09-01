A style that can hold `properties` entries without growing.

`new()` starts empty, and a full resolve then inserts one entry for nearly every
member of `COMPUTED_PROPERTIES` — about a hundred. A map that starts empty
reaches that size through six reallocations, each of which rehashes everything
inserted so far. Paid once per node per restyled frame, on a tree of a few
hundred nodes, that is the largest constant in the style phase.

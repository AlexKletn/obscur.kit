Returns whether a change to the node's active pseudo-states affects it.

True when the flags that `changed` toggled intersect the states this
node's matching depends on; a state the node never matched on cannot alter
its style.

§2.12: "- named slots;" (the partitioning half)

WEAKENED 2026-07-19: this originally also asserted that the two children
carrying no `slot=` assignment land in an unnamed DEFAULT slot, in written
order (`children(default).len() == 2`, classes `["d1", "d2"]`). §2.12's list
is "- named slots; - allowed child contracts; - slot cardinality; - exported
style parts; - exported visual states." — a default/unnamed slot is nowhere
in it, and the case's own `ViewNode::slot("")` outlet is an invention of the
test. The named-slot half, which the spec does state, is kept. Candidate spec
amendment: state whether a component may declare a default (unnamed) slot,
how it is spelled, and where unassigned children go when it is absent.

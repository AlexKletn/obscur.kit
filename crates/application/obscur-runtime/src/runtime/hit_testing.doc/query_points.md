The points the broad phase must probe to surface every candidate.

Each node is tested at its OWN folded point, but the spatial index can only
be asked about one point at a time — so it is asked about all of them: the
screen point, plus one per scroll container carrying a non-zero total fold.
Every node's folded point is `screen` plus the total fold of some container
chain, and each container contributes its own chain's total here, so the
set is complete. A frame with nothing scrolled probes once, exactly as
before scroll containers existed.

Which probe surfaces a node is irrelevant to the answer: the narrow phase
re-tests it at its own point, and a node surfaced twice is judged once.

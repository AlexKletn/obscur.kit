The Tab ring: which nodes keyboard focus visits, and in what order.

Built on demand rather than kept as frame state. A Tab press is rare — it is a
human pressing a key, not a device reporting at its own rate — so the walk
costs nothing anyone can perceive, and a cached ring would need invalidating on
every structural change to answer no better.

The sequence is DOCUMENT order, from a depth-first walk of the retained tree,
and it is deliberately NOT the same set of nodes a pointer can focus. A press
focuses whatever it hit — a plain box, a text leaf — which is both what
spec_ch2_interaction pins and what a user expects from a click. Tab visiting
that set would walk them through every element on the screen.

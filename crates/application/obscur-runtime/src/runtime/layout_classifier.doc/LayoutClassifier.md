Step 8's classifier: given what the style phase actually produced, which
layout contexts can have moved?

A **borrow bundle** over the three things a "has anything moved?" question has
to compare against — the inputs the retained boxes were arranged under, the
styles they were laid out from, and the pre-overlay cascade values the motion
overlay stashed — held for one classification and released with it.

# Why this is an entity and not a branch inside the frame

Its interface is one call returning one of three answers; its internals are a
four-gate conservatism ladder (plan, tree, reason allowlist, non-style inputs)
followed by a per-node property diff whose *candidate set* depends on how the
styles were produced — which is the whole reason [`StyleDelta`] exists. The
two halves speak different vocabularies: the ladder is about frame categories,
the diff is about `PropertyId::affects_layout`, and the only thing joining them
is that both may only ever *remove* provably redundant work.

The invariant it owns is that conservatism. Every gate below returns
[`LayoutPhase::Full`], never `Skip`: a wrong `Full` costs a frame's measure, a
wrong `Skip` paints last frame's geometry and nothing anywhere reports it.

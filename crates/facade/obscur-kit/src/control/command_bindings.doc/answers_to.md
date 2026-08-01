Whether `node` is still the node `target` names.

The cheap half of resolution: [`SelectorResolver`] answers "which node carries
this name" by walking the tree, and this answers "does *this* node still carry
it" by reading one node's properties. A stable identifier lives in the retained
property list rather than in the style attributes — that is what keeps §7.5's "no
effect on matching, layout or visuals" true — so this reads it from there, exactly
as the resolver does.

A [`Coordinate`](SelectorTarget::Coordinate) never names a node (hit-testing is
the runtime's job on the real input path), so it never answers.

Whether `state` holds of `node` or of any of its element ancestors, given
each ancestor's two reserved flags.

The walk is [`AncestorChain`]'s — node-first, following logical parents and
bounded by the node count, so a malformed parent chain terminates. A node
the tree cannot resolve contributes nothing rather than defaulting either
way: an id that is not in this frame's tree is not evidence of a disabled
control.

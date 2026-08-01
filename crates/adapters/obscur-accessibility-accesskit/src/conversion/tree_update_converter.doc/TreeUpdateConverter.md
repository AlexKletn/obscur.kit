Builds a full AccessKit [`TreeUpdate`] from an immutable
[`SemanticSnapshot`] (spec §7.1, §2.7).

The converter walks every non-ignored node in the snapshot, converts each
with its [`NodeConverter`], and assembles the [`TreeUpdate`] that a host
feeds to a platform adapter. A snapshot without a root cannot form a valid
AccessKit tree, so [`convert`](Self::convert) fails with
[`AccessKitBridgeError::MissingRoot`].

# Tree structure

Structure comes from the snapshot's own [`children`](SemanticSnapshot::children)
links — the captured parent→child links of the live [`SemanticTree`] — walked
from the root. An **ignored** node (decorative or hidden) publishes no
AccessKit node, so it is spliced out of the chain and its own children graft
onto its nearest non-ignored ancestor rather than disappearing with it.

A node's [`SemanticRelation::Owns`] relations are honoured on top of that, for
the non-structural ownership a flat projection expresses instead of links (an
`aria-owns`-style reparent); a link and an `Owns` naming the same target agree,
because a target is attached at most once, and the links claim first. Any
non-ignored node that is neither the root nor claimed by either is attached
directly to the root, so the resulting tree is always fully connected as
AccessKit requires.

# Known imprecision

Grafting is right for a **decorative** node — its content is not decorative
just because its wrapper is — but [`is_ignored`](SemanticNode::is_ignored)
conflates decorative with **hidden**, and a hidden node's children are grafted
too. They should be pruned: a subtree under a hidden node is not visible and
should not be announced. It survives here because the projection marks only the
hidden node itself (`SemanticState::hidden` is per-node, not inherited), so this
converter cannot tell a hidden node's child from an ordinary one. The
pre-existing orphan fallback exposed them the same way; splicing does not make
it worse, and does not fix it.

[`SemanticTree`]: obscur_accessibility::tree::SemanticTree
[`SemanticNode`]: obscur_accessibility::semantic::SemanticNode

Builds an AccessKit [`Node`] from a domain [`SemanticNode`] and its child
ids (spec §7.1, §2.7).

The converter composes a [`RoleConverter`] and an [`ActionConverter`], both
stateless, and projects a semantic node's role, accessible name and
description, state flags, value, supported actions, on-screen bounds, and
children onto the corresponding AccessKit properties. Because AccessKit
0.17 builds nodes directly (there is no separate builder type), the returned
[`Node`] is ready to place into a [`accesskit::TreeUpdate`].

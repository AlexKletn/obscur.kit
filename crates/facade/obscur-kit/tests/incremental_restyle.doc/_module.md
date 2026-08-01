End-to-end coverage for audit 3-A (§3.4 phase 5, §4.3): a hover restyles the
nodes it can affect, and **not** the rest of the tree.

The assertions here count work rather than observe output. "The frame rendered"
and "the hovered node turned red" both pass against a full restyle, so they
prove nothing about invalidation; the number of nodes the style phase resolved
separates the two designs. That count is read off the runtime's own §7.9
telemetry — the `style.resolved` counter the style phase records — through a
sink installed via the public facade.

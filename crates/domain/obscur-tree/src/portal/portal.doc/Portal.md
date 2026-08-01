Represents one portal (spec §4.14).

A portal changes where a node is *painted* — its visual parent / stacking
position — while **preserving its logical parent**, and therefore its style
and theme scope. It pairs a **source** node (whose logical position is
unchanged) with a **target** ([`PortalTarget`]) naming the visual
destination host.

# Invariant (spec §4.14)

Applying a portal moves **only** the source node's visual parent to the
target host; its logical parent is left exactly as it was. Because logical
parentage defines style/theme scope, the portalled node keeps inheriting from
its original place in the tree even though it paints under the host.

Identifies where a portal renders its content — the **visual destination**.

A `PortalTarget` is a newtype over the destination **host** [`NodeId`]: the
node that will become the portalled content's visual parent (its "portal
host", spec §4.14). It says only *where* portalled content is painted; it
carries no styling and does not change the source node's logical parent.

For Phase 1 a target is a single host node. Richer targets — a named window,
an overlay layer, or a resolved-by-name slot — are a deliberate later
extension and are intentionally not modelled here.

A stable identifier for a [`GestureRecognizer`] within a gesture arena
(spec §6.16).

This is a local newtype over an opaque `u64`; it is distinct from
[`NodeId`](obscur_identity::NodeId) and only carries meaning within the
interaction layer's gesture arena.

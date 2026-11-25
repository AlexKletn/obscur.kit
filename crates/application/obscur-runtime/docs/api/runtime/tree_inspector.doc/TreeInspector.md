The developer-facing **projection** of what a frame produced (spec §7.6): the
structured tree snapshot an agent asserts on, the readout for the node under
the pointer, and the in-window highlight overlay.

It holds one piece of runtime state — whether the overlay is on — and takes
everything it reads as arguments: the retained tree, the computed styles, the
arranged boxes, the §4.16 generated content and the live scroll offsets all
belong to the frame pipeline, and this is a reader of what that pipeline left
behind.

# Why this is one entity and not three accessors

The three outputs answer one question — *what is on screen right now, named
the way a developer names it* — and they answer it in a vocabulary nothing
else in the runtime speaks: a **projected** node that is not a tree node.
[`snapshot`](Self::snapshot) appends synthetic nodes for the scrollbar parts
and the placeholder, which exist in no node's `children()` and have ids minted
out of a reserved high-bit space so they cannot collide with a real
[`NodeId`]. That id-space discipline is the entity's own rule: the two part
families take `1 << 62` and `1 << 61`, and a third family added anywhere else
would silently alias one of them.

The interface is three calls; the internals are a pre-order walk, two
synthetic-part projections, a fixed style digest and an overlay append.

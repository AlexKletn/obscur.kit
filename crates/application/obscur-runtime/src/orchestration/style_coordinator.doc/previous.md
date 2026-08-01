The style each re-resolved node had **before**, for those that had one.

Together with [`introduced`](Self::introduced) this is the complete answer to
"where can the map have changed?": a node absent from both was not
re-resolved, so it provably still holds the style it did last frame.

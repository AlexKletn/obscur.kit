What the resolver is told about the document a node sits in:
[`ResolutionContext`].

Every relative unit folds against a base. `em`'s base is the node's own
`font-size`, which the resolver computes for itself; the other three cannot be
derived from one node's cascade at all — the **root**'s font-size (`rem`), the
**viewport** (`vw` / `vh` / `vmin` / `vmax`) and a measurement of the `0` glyph
(`ch`) — so they arrive from the caller, in this one value.

It is a separate entity from the resolver because it is the resolver's
*input*: one is built per document per frame and handed to every node, while
the resolver itself is stateless.

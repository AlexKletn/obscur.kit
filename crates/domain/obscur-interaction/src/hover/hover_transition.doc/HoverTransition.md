The enter/leave diff between a previous and a new [`HoverChain`] (hover
enter/leave).

Computed by [`between`](Self::between): `entered` holds the nodes newly under
the pointer (in the new chain but not the old), `left` holds the nodes no
longer under the pointer (in the old chain but not the new). Each list
preserves the order of the chain it was drawn from. These drive standard
pointer enter/leave semantics: an enter fires for each newly-hovered node, a
leave for each exited node.

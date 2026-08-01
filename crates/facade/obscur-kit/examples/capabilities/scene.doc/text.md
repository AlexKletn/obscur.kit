A classed text run: a classed [`Container`] **element** wrapping a bare text
node. The class carries `color` / `font-size` / spacing / `white-space` /
`text-overflow` on the element (all inherited or box-level properties), and the
text node inherits them — the engine matches class selectors against elements,
not against text nodes, and resolves a run's color/size/white-space from the
containing element, so styling the wrapper is what actually paints.

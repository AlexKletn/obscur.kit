Reconciles `desired` into the retained tree and returns
`(changed, restyle_roots)`.

`changed` is whether the tree actually moved — the root commit carried an
operation, or a component sub-render did. `restyle_roots` are the subtrees
the style phase can restrict itself to; [`None`] means the edit could not be
bounded and a full pass is the
only correct answer.

`viewport_scroll_y` is the surface's current vertical scroll, which the
§4.9 realization reads to choose each list's window.

`previous_sheets` supplies the previous cascade’s child-structure dependencies.
The style phase falls back to a full pass if the cascade version changes.

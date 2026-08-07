[`shape_paragraph`](Self::shape_paragraph) without the copy: hands back the
memoized [`ShapedRun`]s themselves, shared.

A cache hit is then a refcount bump instead of a deep clone of every glyph
in the paragraph. That matters because the hit path is the frame path — a
scrolling list re-measures and re-composes the same visible runs sixty times
a second, and each of those calls was allocating a fresh `Vec<ShapedRun>`
plus a fresh glyph buffer per run only to read advances off it.

`shape_paragraph` remains the copying form for callers that need to own and
mutate the runs (folding letter-/word-spacing into the advances, say).

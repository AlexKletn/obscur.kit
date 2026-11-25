[`shape_paragraph`](Self::shape_paragraph) without the copy: hands back the
memoized [`ShapedRun`]s themselves, shared.

A cache hit is then a refcount bump instead of a deep clone of every glyph
in the paragraph. That matters because the hit path is the frame path — a
scrolling list re-measures and re-composes the same visible runs sixty times
a second, and each of those calls was allocating a fresh `Vec<ShapedRun>`
plus a fresh glyph buffer per run only to read advances off it.

This is the form every in-frame caller uses — the compose pass, the measure
pass and the caret geometry alike. `shape_paragraph` remains the copying form,
kept for callers that want an owned `Vec` (the crate's own tests do), and it is
no longer on any frame path.

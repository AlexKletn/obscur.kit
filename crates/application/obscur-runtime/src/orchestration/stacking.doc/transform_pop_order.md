The phase for a subtree's closing `PopTransform` at its *exit* index. Uses
the same depth-descending schedule as [`clip_pop_order`](Self::clip_pop_order)
so nested transforms unwind deepest-first; a transform and a clip that end
at the same index and depth tie, broken by emitting the transform bracket
**after** the clip bracket (so the clip closes first, inside the transform).

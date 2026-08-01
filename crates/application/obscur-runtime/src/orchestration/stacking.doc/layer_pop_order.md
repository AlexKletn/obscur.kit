The phase for a subtree's closing `PopLayer` (a CSS `opacity` group
layer) at its *exit* index. Uses the same depth-descending schedule as
[`clip_pop_order`](Self::clip_pop_order) so nested layers unwind
deepest-first; a layer, a clip, and a transform that end at the same index
and depth tie, broken by emitting the brackets in `clip, layer, transform`
order so the clip closes first (innermost), then the layer, then the
transform (outermost).

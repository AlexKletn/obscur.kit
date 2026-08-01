The phase for a subtree's closing `PopLayer` (a CSS `filter: blur` group
layer) at its *exit* index. Uses the same depth-descending schedule as
[`clip_pop_order`](Self::clip_pop_order); a blur, transform, opacity, and
clip that end at the same index and depth tie, broken by emitting the blur
bracket **last** so it closes after them all (the blur is the outermost
wrapper).

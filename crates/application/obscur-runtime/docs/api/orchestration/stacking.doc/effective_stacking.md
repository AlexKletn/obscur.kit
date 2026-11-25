Every node's effective stacking `z` and CSS 2.1 §9.9 paint-layer rank, in one
pre-order walk.

`z` is a positioned element's numeric `z-index`, else its nearest z-index
ancestor's, with the root at 0. `layer`, sorted between `z` and the pre-order
index, lifts a positioned `z: auto`/`0` box and its inherited subtree above
non-positioned siblings in the same `z` bucket. It is uniformly
`NonPositioned` when nothing is positioned, so the sort then reduces to
`(z, tree_index)`.

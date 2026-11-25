The desired node's authored children, kept for §2.12 slot projection.

Retained for the same reason [`props`](Self::props) is — the desired tree
is gone by the time a component renders, and a component's output declares
*where* its authored children go (see
[`ViewNode::slot`](obscur_view::view::ViewNode::slot)), so the runtime must
be able to look the content up again after the frame's reconcile.

Populated **only** when at least one authored child carries a `slot="…"`
assignment: keeping every element's children would clone the desired
subtree once per level of depth, which is a cost a document that authors no
slot must not pay. Empty for every other node, which is nearly all of them.

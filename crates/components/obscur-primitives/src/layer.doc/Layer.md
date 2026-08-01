A compositing layer that groups children with an applied opacity.

`Layer` is one of the minimal built-in primitives (spec §2.6). It maps to a
scene compositing layer (spec §5.1): its children are rendered into an
isolated layer that is then composited with the layer's `opacity`, so
group-level transparency and blending apply as one unit. It is a leaf
element primitive: it lowers to a [`ViewNode::Element`] carrying its opacity
as a property and its children as the element's fragment.

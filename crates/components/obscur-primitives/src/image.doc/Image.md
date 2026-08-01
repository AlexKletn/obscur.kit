A reference to an image resource drawn into the view tree.

`Image` is one of the minimal built-in primitives (spec §2.6). It names an
image by an opaque `source` key that the resource hub resolves to real pixel
or vector data (spec §5.16), and optionally an `alt` accessible name. It is
a leaf element primitive: it lowers to a [`ViewNode::Element`] carrying its
source (and alt, when present) as properties.

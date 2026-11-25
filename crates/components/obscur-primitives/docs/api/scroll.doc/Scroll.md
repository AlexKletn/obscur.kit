A scrollable viewport wrapping a set of children.

`Scroll` is one of the minimal built-in primitives (spec §2.6). It clips its
children to a viewport and offers scrolling along the enabled axes
(spec §4.8). By default it scrolls vertically only; either axis can be
toggled. It is a leaf element primitive: it lowers to a
[`ViewNode::Element`] carrying its per-axis flags as properties and its
children as the element's fragment.

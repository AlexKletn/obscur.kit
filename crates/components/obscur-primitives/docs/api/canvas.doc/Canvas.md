A custom-rendering surface that participates in layout and hit-testing.

`Canvas` is one of the minimal built-in primitives (spec §2.6). It reserves
a region the application paints itself (spec §5.3) and carries an intrinsic
size hint so the layout engine can size it before the custom paint runs. It
is a leaf element primitive with no children: it lowers to a
[`ViewNode::Element`] carrying its width and height as properties.

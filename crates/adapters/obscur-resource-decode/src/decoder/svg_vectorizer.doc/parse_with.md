Parses `bytes` with `color` injected as the root's `color` property.

The stylesheet targets `svg` rather than `*` deliberately: it seeds the root
so `currentColor` inherits down, while an element that declares its own colour
still wins for its own subtree.

A `.page` **root** wrapping a `.box` that wraps a text node.

The nesting is load-bearing for the `font-size` tests: `.box` must have a real
parent, or `em` (which means the parent's size) and `rem` (which means the
root's) would collapse onto the same node and agree by accident.

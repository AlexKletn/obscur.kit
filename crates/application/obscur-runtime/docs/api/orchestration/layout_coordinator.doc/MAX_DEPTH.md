The maximum recursion depth of the layout walk, including the synthetic root.

This bounds both measurement and arrangement. At the limit, measurement
returns zero and arrangement omits the remaining boxes. The runtime emits
a `layout.depth_limit` diagnostic when the pass reaches this boundary.

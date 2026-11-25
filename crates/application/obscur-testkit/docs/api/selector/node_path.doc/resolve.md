The node at the end of the path.

# Panics

If the tree has no root — which means no frame has run — or if any step names a
child that is not there. Both messages name the path and the depth that failed.

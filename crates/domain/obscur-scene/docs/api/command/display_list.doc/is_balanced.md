Checks that transform, clip, and layer commands open and close in nested order.
Each pop must match the most recent open scope, and every push must be closed
by the end of the list.

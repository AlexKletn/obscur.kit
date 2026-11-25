Converts `snapshot` into a full [`TreeUpdate`] focused on its root.

Equivalent to [`convert_with_focus`](Self::convert_with_focus) with no
explicit focus target, so focus defaults to the root as AccessKit
requires when no node is focused.

# Errors

Returns [`AccessKitBridgeError::MissingRoot`] when the snapshot has no
root node.

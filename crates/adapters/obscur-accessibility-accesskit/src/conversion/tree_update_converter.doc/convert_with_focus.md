Converts `snapshot` into a full [`TreeUpdate`], focusing `focus` when it
names a present, non-ignored node and falling back to the root
otherwise.

# Errors

Returns [`AccessKitBridgeError::MissingRoot`] when the snapshot has no
root node.

Returns whether the inputs actually advanced since the last update.

When this is `false` the hook may skip work entirely, matching the
spec's "invalidate only consumers of changed values".

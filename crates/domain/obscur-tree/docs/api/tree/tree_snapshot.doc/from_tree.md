Builds a snapshot from the current state of `tree`.

Clones every live node record, captures the root, and records the tree's
current [`VersionStamp`]. Prefer [`RetainedTree::snapshot`], which calls
this.

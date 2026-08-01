Rebuilds a parent→children map from the snapshot's own child links, adds
the ownership its `Owns` relations express, then attaches every
still-parentless non-ignored node to the root.

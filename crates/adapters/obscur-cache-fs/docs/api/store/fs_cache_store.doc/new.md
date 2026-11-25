Creates a store rooted at `root`, creating the directory (and any missing
parents) if it does not already exist.

Reads miss while the directory is unavailable. A write recreates a missing
directory before retrying. If the directory remains unavailable, the write is
skipped.

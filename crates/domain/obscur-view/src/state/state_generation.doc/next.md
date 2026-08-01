Returns the next generation, advancing the monotonic counter by one.

Wraps on overflow, matching the backing [`VersionStamp`]; a `u64` counter
makes real-world wraparound irrelevant.

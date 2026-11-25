Locks the queue, recovering from a poisoned lock.

A job that panics while the queue is *unlocked* cannot poison it, but a
panic elsewhere holding the guard would; recovering keeps a test's failure
message the original panic rather than a confusing poison error.

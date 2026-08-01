Cancels the provisional transaction, discarding all staged operations
without producing a commit.

This is intentionally free: staging never mutated any domain, so cancel
only needs to drop the pending transaction. Consuming `self` guarantees a
cancelled transaction cannot later be committed.

Increments the content version to signal that the Canvas content changed.

Saturates at [`u64::MAX`] rather than wrapping, so the version never
appears to move backwards.

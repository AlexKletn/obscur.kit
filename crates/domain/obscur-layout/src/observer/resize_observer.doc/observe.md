Records the newly-committed size, returning a [`ResizeChange`] when it
differs from the last observed size (or on the first observation).

Returns `None` when the size is unchanged. Updates the stored size on a
fire.

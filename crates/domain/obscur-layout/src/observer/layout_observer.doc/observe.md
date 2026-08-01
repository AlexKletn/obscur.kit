Records the newly-committed box, returning a [`LayoutChange`] when it
differs from the last observed box (or on the first observation).

Returns `None` when the box is unchanged. Updates the stored box on a
fire.

Records the newly-committed transform, returning a [`TransformChange`]
when it differs from the last observed transform (or on the first
observation).

Returns `None` when the transform is unchanged. Updates the stored
transform on a fire.

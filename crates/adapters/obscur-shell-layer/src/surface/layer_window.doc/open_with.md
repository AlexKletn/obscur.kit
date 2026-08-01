[`open`](Self::open), plus a [`LayerWaker`] the loop will answer to.

# Errors

The same as [`open`](Self::open), plus [`LayerShellError::Dispatch`] if
the waker's source cannot be inserted into the loop.

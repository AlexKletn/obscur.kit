Binds a Unix domain listener to `path`.

A filesystem-scoped socket is local by construction. The caller owns the
path's lifetime (a temp directory keeps tests hermetic).

# Errors

Returns an [`io::Error`] if the path is already bound or the bind fails.

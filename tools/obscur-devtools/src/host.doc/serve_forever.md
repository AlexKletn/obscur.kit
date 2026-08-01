Serves client sessions forever (one connection at a time), each over a fresh
clone of the demo source.

# Errors

Returns an [`io::Error`] on any accept/serve socket failure.

A refcounted handle to the same pixels, for a backend that wants to hold them
without copying — the Vello adapter wraps this straight into a texture blob.

Cloning it is a refcount bump whichever shape the bytes are in.

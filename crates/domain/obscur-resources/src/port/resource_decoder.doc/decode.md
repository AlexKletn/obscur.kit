Decodes `bytes` of the given `kind` into a CPU representation (spec
§5.15).

# Errors

Returns [`DecodedResource`] on success, or a [`ResourceError`] describing
the failure — [`ResourceError::Unsupported`] when this decoder handles no
such kind, or [`ResourceError::DecodeFailed`] when the bytes are
malformed.

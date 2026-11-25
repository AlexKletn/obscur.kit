Compiles `decoded` of the given `kind` into a backend/GPU artifact (spec
§5.15), returning its [`CompiledResource`] descriptor.

# Errors

Returns a [`ResourceError`] on failure — [`ResourceError::Unsupported`]
for a kind this compiler does not handle, or
[`ResourceError::CompileFailed`] when compilation itself fails.

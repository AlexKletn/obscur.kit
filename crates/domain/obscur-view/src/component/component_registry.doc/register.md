Registers a component kind.

# Errors

Returns [`RegistryError::DuplicateComponent`] (carrying the offending
[`ComponentId`]) if a registration with the same id already exists; the
existing entry is left untouched. This never panics.

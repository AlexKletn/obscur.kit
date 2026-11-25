Decodes `props` into `Self`.

# Errors

Returns [`PropsError::Missing`] when a required property is absent, and
[`PropsError::Type`] when one is present but cannot be read as the
field's type.

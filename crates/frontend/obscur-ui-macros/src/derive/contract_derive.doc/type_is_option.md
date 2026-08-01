Returns whether `ty` is syntactically an `Option<...>`.

This is a purely textual check on the last path segment, matching `Option`,
`std::option::Option` and `core::option::Option`.

Creates a new window from `request`, minting and returning its
[`WindowId`] (spec §6.1).

# Errors

Returns [`WindowError`] when creation cannot be satisfied; a backend that
cannot create windows at all returns [`WindowError::Unsupported`] rather
than panicking.

Fetches `request` and returns the host's [`NetworkResponse`]
(spec §5.15).

A service that performs no network access returns
[`NetworkError::Unsupported`] rather than panicking.

# Errors

Returns a [`NetworkError`] when the fetch cannot be satisfied (denied,
timed out, unsupported, or a transport failure).

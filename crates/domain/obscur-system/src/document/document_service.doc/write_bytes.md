Writes `bytes` to the document behind `handle` (spec §6.10).

# Errors

Must return [`DocumentError::PermissionDenied`] when the handle is not
writable ([`DocumentHandle::can_write`]).

Creates a request for `source` of the given `kind`, owned by the implicit
global namespace (spec §5.15).

Use [`ResourceRequest::in_namespace`] to scope it to a specific
[`ResourceNamespace`].

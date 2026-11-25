Appends a `name: value` header, preserving order (fluent; spec §5.15).

Existing headers are never replaced, so repeated names accumulate in the
order they were added — matching HTTP's multi-valued header semantics.

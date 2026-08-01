Invalidates `child` if `dep` shows it consumed something the parent's
change (`parent_diff`) altered (§4.14).

Consults [`StyleDependency::affected_by_parent_change`] and records the
returned reason; a node the parent change does not affect is left
untouched. Returns whether `child` was invalidated by this call. The caller
(the runtime) is responsible for having identified `child` as a child of
the changed parent.

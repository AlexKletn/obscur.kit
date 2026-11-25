Attempts to match `route`'s path against `pattern` (spec §6.13).

Returns `Some(RouteMatch)` with the captured parameters and the route's
path when the pattern matches, or `None` otherwise. Only the path
participates in matching; the route's query and fragment are ignored.

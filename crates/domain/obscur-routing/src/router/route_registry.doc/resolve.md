Resolves `route` to the first matching registration (spec §6.13).

Returns the [`RouteMatch`] (captured params and matched path) paired with
the target id of the first registered pattern that matches `route`, or
`None` when no pattern matches (a deep-link miss). Registration order is
priority: the first match wins.

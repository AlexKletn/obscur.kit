The outcome of successfully matching a [`Route`] against a [`RoutePattern`]
(spec §6.13).

A `RouteMatch` bundles the [`RouteParams`] captured by the pattern with the
concrete path that matched, giving a caller everything needed to resolve the
route to a screen or component without re-running the match.

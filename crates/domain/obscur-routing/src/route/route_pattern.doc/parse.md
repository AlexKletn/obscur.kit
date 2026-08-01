Compiles a pattern string into a [`RoutePattern`] (spec §6.13).

The string is split on `/` and empty components are skipped, so leading
and trailing slashes are irrelevant. Each remaining component becomes a
segment: `:name` is a [`Param`](PatternSegment::Param), `*name` is a
[`Wildcard`](PatternSegment::Wildcard), and anything else is a
[`Static`](PatternSegment::Static) literal. A pattern with no segments
(for example `/`) matches only the root path.

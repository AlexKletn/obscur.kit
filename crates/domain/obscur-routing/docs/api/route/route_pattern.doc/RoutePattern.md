A compiled route pattern such as `/users/:id/posts/:postId` or
`/files/*rest` (spec §6.13).

A pattern is a sequence of [`PatternSegment`]s matched against a path's
segments by [`match_path`](RoutePattern::match_path). Nested routes are
expressed simply as longer patterns (deeper paths) or a trailing
[`Wildcard`](PatternSegment::Wildcard) that absorbs an arbitrarily deep
tail; the toolkit does not maintain a separate route tree.

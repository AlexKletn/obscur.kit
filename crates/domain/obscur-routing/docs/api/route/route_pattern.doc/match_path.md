Matches `path` against the pattern, returning the captured
[`RouteParams`] on success (spec §6.13).

The path is split into non-empty segments and consumed left to right
against the pattern segments:

- [`Static`](PatternSegment::Static) must equal the current path
  segment.
- [`Param`](PatternSegment::Param) captures one path segment under its
  name.
- [`Wildcard`](PatternSegment::Wildcard) captures *all* remaining path
  segments (re-joined with `/`) under its name and ends the match; a
  wildcard with no remaining segments captures the empty string.

A match succeeds only when the pattern and path are consumed in step:
every path segment must be accounted for (unless a wildcard absorbed the
tail) and every non-wildcard pattern segment must have a path segment.
Returns `None` on any mismatch. An empty pattern matches only a path
with no segments (the root `/`).

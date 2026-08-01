§6.13: "- path;" (URL-like) with "- typed params;"

An encoded slash stays inside one segment: it does not become a segment
break, and it does not shift the following segments.

WEAKENED 2026-07-19: this originally asserted that `%2F` is DECODED —
`params.get("name") == Some("a/b")`, and that `%252F` decodes exactly once to
`a%2F`. §6.13's complete text is "Basic URL-like router:" followed by
the eleven bullets "path", "query", "fragment", "route patterns", "typed
params", "nested routes", "history", "back/forward", "deep-link mapping",
"typed route state" and "route-to-screen/component mapping". It never
mentions percent-encoding,
decoding, or encoding, so both the decode and the exactly-once ordering were
derived from URL conventions rather than from this spec. What is kept is the
half that a router must get right whatever its decoding policy, and that
still discriminates: an implementation that decodes BEFORE segmenting splits
`a%2Fb` into two segments, the pattern stops matching, and `:action` binds
`b` instead of `edit`. Candidate spec amendment: state whether the router
percent-decodes path segments and query values at all, and if so that it
decodes after segmentation and exactly once — the two failure modes are a
silently truncated param and a double-decode injection.

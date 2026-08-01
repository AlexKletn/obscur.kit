Returns the non-empty path segments, splitting on `/`.

Empty segments (from leading, trailing, or doubled slashes) are skipped,
so `/a/b/` yields `["a", "b"]`.

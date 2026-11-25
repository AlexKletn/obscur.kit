The ordinary, **unforgiving** selector list `:not()` and `:has()` take, and
the reason they cannot share the forgiving path.

Returning [`None`] for an unparsable argument is not a stylistic choice: an
empty `:not()` list matches *every element*, because `!any([])` is `!false`.
So the tolerant reading fails **open** — `:not(em, %%%)` would silently drop
the valid `em` branch and turn "everything except em" into "everything,
unconditionally", which is the opposite of what the author wrote. The caller
maps [`None`] to an unrecognised pseudo-class, which matches nothing.

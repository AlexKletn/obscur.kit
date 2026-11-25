Creates a range spanning `start..end`.

The range is normalized: if `end < start` the end is clamped up to
`start` (`end.max(start)`), yielding an empty range at `start` rather
than an inverted one. A `debug_assert` flags inverted input in debug
builds.
